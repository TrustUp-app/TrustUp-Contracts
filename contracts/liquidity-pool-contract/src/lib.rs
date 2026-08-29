#![no_std]
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, Env, IntoVal, Symbol,
};

mod access;
mod errors;
mod events;
mod storage;
mod types;

pub use errors::LiquidityPoolError;
pub use types::{PoolStats, RateParams};

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolContract {
    // -------------------------------------------------------------------------
    // Initialization
    // -------------------------------------------------------------------------

    /// Initialize the contract. Can only be called once.
    ///
    /// * `admin`        – Contract administrator (can update addresses)
    /// * `token`        – SEP-41 token used by the pool (e.g. USDC)
    /// * `treasury`     – Address that receives the 10% protocol fee
    /// * `merchant_fund`– Address that receives the 5% merchant incentive fee
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        treasury: Address,
        merchant_fund: Address,
    ) {
        if storage::has_admin(&env) {
            panic_with_error!(&env, LiquidityPoolError::AlreadyInitialized);
        }
        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
        storage::set_treasury(&env, &treasury);
        storage::set_merchant_fund(&env, &merchant_fund);

        // Seed the default utilization-based rate curve so the pool always has a
        // well-formed model; governance can retune it later via `set_rate_params`.
        storage::set_rate_params(
            &env,
            &types::RateParams {
                base_rate_bps: types::DEFAULT_BASE_RATE_BPS,
                slope1_bps: types::DEFAULT_SLOPE1_BPS,
                slope2_bps: types::DEFAULT_SLOPE2_BPS,
                optimal_utilization_bps: types::DEFAULT_OPTIMAL_UTILIZATION_BPS,
            },
        );
    }

    // -------------------------------------------------------------------------
    // Admin setters
    // -------------------------------------------------------------------------

    pub fn set_creditline(env: Env, admin: Address, creditline: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_creditline(&env, &creditline);
    }

    pub fn set_treasury(env: Env, admin: Address, treasury: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_treasury(&env, &treasury);
    }

    pub fn set_merchant_fund(env: Env, admin: Address, merchant_fund: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_merchant_fund(&env, &merchant_fund);
    }

    pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_admin(&env, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    pub fn set_parameters_contract(env: Env, admin: Address, address: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_parameters_contract(&env, &address);
    }

    pub fn get_parameters_contract(env: Env) -> Option<Address> {
        storage::get_parameters_contract(&env)
    }

    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_paused(&env, true);
    }

    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_paused(&env, false);
    }

    pub fn is_paused(env: Env) -> bool {
        if storage::is_paused(&env) {
            return true;
        }
        if let Some(params_addr) = storage::get_parameters_contract(&env) {
            let is_paused: bool = env
                .try_invoke_contract::<bool, soroban_sdk::Error>(
                    &params_addr,
                    &Symbol::new(&env, "is_paused"),
                    ().into_val(&env),
                )
                .unwrap_or(Ok(false))
                .unwrap_or(false);
            if is_paused {
                return true;
            }
        }
        false
    }

    // -------------------------------------------------------------------------
    // Utilization-based interest rate model (Aave-style kinked curve)
    // -------------------------------------------------------------------------

    /// Governance-gated update of the rate curve parameters. All values are bps
    /// (10000 = 100%). `optimal_utilization_bps` must be strictly inside
    /// (0, 10000); slopes and base must be non-negative.
    pub fn set_rate_params(
        env: Env,
        admin: Address,
        base_rate_bps: i128,
        slope1_bps: i128,
        slope2_bps: i128,
        optimal_utilization_bps: i128,
    ) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        if base_rate_bps < 0
            || slope1_bps < 0
            || slope2_bps < 0
            || optimal_utilization_bps <= 0
            || optimal_utilization_bps >= types::TOTAL_BPS
        {
            panic_with_error!(&env, LiquidityPoolError::InvalidRateParams);
        }

        storage::set_rate_params(
            &env,
            &types::RateParams {
                base_rate_bps,
                slope1_bps,
                slope2_bps,
                optimal_utilization_bps,
            },
        );
        events::emit_rate_params_updated(
            &env,
            base_rate_bps,
            slope1_bps,
            slope2_bps,
            optimal_utilization_bps,
        );
        storage::bump_instance(&env);
    }

    /// Current rate curve parameters (defaults if never set).
    pub fn get_rate_params(env: Env) -> RateParams {
        Self::rate_params(&env)
    }

    /// Current pool utilization in bps: `locked_liquidity / total_liquidity`.
    /// Returns 0 for an empty pool.
    pub fn get_utilization_bps(env: Env) -> i128 {
        Self::utilization_bps_internal(&env)
    }

    /// Borrow rate (bps) for the pool's *current* utilization.
    pub fn get_current_rate_bps(env: Env) -> i128 {
        let params = Self::rate_params(&env);
        let utilization = Self::utilization_bps_internal(&env);
        Self::compute_rate_bps(&env, &params, utilization)
    }

    /// Borrow rate (bps) at a hypothetical `utilization_bps` (0..=10000), using
    /// the current curve. Pure view — useful for quoting and off-chain UIs.
    pub fn quote_rate_bps(env: Env, utilization_bps: i128) -> i128 {
        let params = Self::rate_params(&env);
        Self::compute_rate_bps(&env, &params, utilization_bps)
    }

    /// Interest owed on `principal` at the current utilization-based rate, after
    /// applying a reputation-based discount (bps) supplied by `creditline-contract`.
    /// The effective rate is floored at `base_rate` so a discount can never push
    /// borrowing below the base cost. This is the wiring point CreditLine calls
    /// instead of the legacy fixed `base_interest_bps`.
    pub fn quote_interest(env: Env, principal: i128, reputation_discount_bps: i128) -> i128 {
        if principal < 0 || reputation_discount_bps < 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let params = Self::rate_params(&env);
        let utilization = Self::utilization_bps_internal(&env);
        let rate = Self::compute_rate_bps(&env, &params, utilization);

        // Reputation discount lowers the rate, but never below the base rate.
        let effective_rate = (rate - reputation_discount_bps).max(params.base_rate_bps);

        principal
            .checked_mul(effective_rate)
            .and_then(|v| v.checked_div(types::TOTAL_BPS))
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow))
    }

    // -------------------------------------------------------------------------
    // LP Operations
    // -------------------------------------------------------------------------

    /// Deposit `amount` tokens and receive shares representing pool ownership.
    ///
    /// **First deposit**: shares issued == amount (1:1 ratio).
    /// **Subsequent deposits**: `shares = (amount × total_shares) / total_pool_value`
    ///
    /// Returns the number of shares issued.
    pub fn deposit(env: Env, provider: Address, amount: i128) -> i128 {
        provider.require_auth();
        Self::require_not_paused(&env);

        if amount < types::MIN_AMOUNT {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        Self::enter_non_reentrant(&env);

        let token = storage::get_token(&env);
        let total_shares = storage::get_total_shares(&env);
        let total_liquidity = storage::get_total_liquidity(&env);

        // Calculate shares to issue
        let shares_issued = if total_shares == 0 || total_liquidity == 0 {
            // First deposit: 1:1 ratio
            amount
        } else {
            // Subsequent deposits: proportional to current pool value
            amount
                .checked_mul(total_shares)
                .and_then(|v| v.checked_div(total_liquidity))
                .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow))
        };

        if shares_issued <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        // Update state
        let new_shares = storage::get_lp_shares(&env, &provider)
            .checked_add(shares_issued)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_lp_shares(&env, &provider, new_shares);
        storage::bump_lp_shares(&env, &provider);

        let new_total_shares = total_shares
            .checked_add(shares_issued)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_shares(&env, new_total_shares);

        let new_total_liquidity = total_liquidity
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_liquidity(&env, new_total_liquidity);

        // Transfer tokens from provider to pool contract after state effects.
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&provider, &env.current_contract_address(), &amount);

        events::emit_liquidity_deposited(&env, &provider, amount, shares_issued);
        storage::bump_instance(&env);
        Self::exit_non_reentrant(&env);

        shares_issued
    }

    /// Burn `shares` and return the proportional token amount to `provider`.
    ///
    /// `amount = (shares × total_pool_value) / total_shares`
    ///
    /// Returns the number of tokens returned.
    pub fn withdraw(env: Env, provider: Address, shares: i128) -> i128 {
        provider.require_auth();
        Self::require_not_paused(&env);

        if shares <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        Self::enter_non_reentrant(&env);

        let provider_shares = storage::get_lp_shares(&env, &provider);
        if provider_shares < shares {
            panic_with_error!(&env, LiquidityPoolError::InsufficientShares);
        }

        let total_shares = storage::get_total_shares(&env);
        if total_shares == 0 {
            panic_with_error!(&env, LiquidityPoolError::ZeroTotalShares);
        }

        let total_liquidity = storage::get_total_liquidity(&env);
        let locked_liquidity = storage::get_locked_liquidity(&env);
        let available_liquidity = total_liquidity
            .checked_sub(locked_liquidity)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));

        // Calculate withdrawal amount proportionally
        let amount_returned = shares
            .checked_mul(total_liquidity)
            .and_then(|v| v.checked_div(total_shares))
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));

        if amount_returned > available_liquidity {
            panic_with_error!(&env, LiquidityPoolError::InsufficientLiquidity);
        }

        // Burn shares
        let new_provider_shares = provider_shares
            .checked_sub(shares)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_lp_shares(&env, &provider, new_provider_shares);
        storage::bump_lp_shares(&env, &provider);

        let new_total_shares = total_shares
            .checked_sub(shares)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_total_shares(&env, new_total_shares);

        let new_total_liquidity = total_liquidity
            .checked_sub(amount_returned)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_total_liquidity(&env, new_total_liquidity);

        events::emit_liquidity_withdrawn(&env, &provider, shares, amount_returned);
        // Transfer tokens back to provider after state effects.
        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &provider, &amount_returned);
        storage::bump_instance(&env);
        Self::exit_non_reentrant(&env);

        amount_returned
    }

    // -------------------------------------------------------------------------
    // Share Transfers (SEP-41-style secondary market)
    // -------------------------------------------------------------------------
    //
    // LP shares are a fungible, pool-wide claim: a share entitles its holder to
    // `total_liquidity / total_shares` tokens, never to a specific loan. Moving
    // shares between holders therefore leaves `total_shares`, `total_liquidity`
    // and `locked_liquidity` untouched — only the owner of the claim changes.
    // Loan exposure is enforced where value actually leaves the pool
    // (`withdraw`, which is capped by available liquidity), so transfers stay
    // available while loans are outstanding instead of freezing every LP's
    // position. See the contract README for the full rationale.

    /// Allow `spender` to move up to `amount` of `from`'s shares until
    /// `expiration_ledger` (inclusive). Overwrites any previous allowance.
    ///
    /// Passing `amount == 0` revokes an existing allowance.
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();

        if amount < 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let current_ledger = env.ledger().sequence();
        if amount > 0 {
            // A live allowance cannot expire in the past…
            if expiration_ledger < current_ledger {
                panic_with_error!(&env, LiquidityPoolError::InvalidExpirationLedger);
            }
            // …nor outlive the maximum TTL the network will store it for.
            let live_for = expiration_ledger - current_ledger;
            if live_for > env.storage().max_ttl() {
                panic_with_error!(&env, LiquidityPoolError::InvalidExpirationLedger);
            }
        }

        storage::set_allowance(&env, &from, &spender, amount, expiration_ledger);
        events::emit_approval(&env, &from, &spender, amount, expiration_ledger);
        storage::bump_instance(&env);
    }

    /// Shares `spender` may still move on behalf of `from`.
    /// Returns 0 once the allowance has expired.
    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        storage::get_allowance(&env, &from, &spender).amount
    }

    /// Move `amount` shares from `from` to `to`.
    ///
    /// Pool accounting is unchanged: this is a pure reassignment of ownership.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        Self::require_not_paused(&env);

        if amount < types::MIN_AMOUNT {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        Self::enter_non_reentrant(&env);
        Self::move_shares(&env, &from, &to, amount);
        events::emit_share_transferred(&env, &from, &to, amount);
        storage::bump_instance(&env);
        Self::exit_non_reentrant(&env);
    }

    /// Move `amount` shares from `from` to `to`, spending `spender`'s allowance.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        Self::require_not_paused(&env);

        if amount < types::MIN_AMOUNT {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        Self::enter_non_reentrant(&env);
        Self::spend_allowance(&env, &from, &spender, amount);
        Self::move_shares(&env, &from, &to, amount);
        events::emit_share_transferred(&env, &from, &to, amount);
        storage::bump_instance(&env);
        Self::exit_non_reentrant(&env);
    }

    // -------------------------------------------------------------------------
    // CreditLine Operations (access-restricted)
    // -------------------------------------------------------------------------

    /// Transfer `amount` tokens to `merchant` to fund a loan.
    /// Only the registered CreditLine contract may call this.
    pub fn fund_loan(env: Env, creditline: Address, merchant: Address, amount: i128) {
        creditline.require_auth();
        access::require_creditline(&env, &creditline);
        Self::require_not_paused(&env);

        if amount <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        Self::enter_non_reentrant(&env);

        let total_liquidity = storage::get_total_liquidity(&env);
        let locked_liquidity = storage::get_locked_liquidity(&env);
        let available = total_liquidity
            .checked_sub(locked_liquidity)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));

        if amount > available {
            panic_with_error!(&env, LiquidityPoolError::InsufficientLiquidity);
        }

        let new_locked = locked_liquidity
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_locked_liquidity(&env, new_locked);

        // Transfer tokens from pool to merchant after accounting has been updated.
        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &merchant, &amount);

        events::emit_loan_funded(&env, &creditline, amount);
        Self::exit_non_reentrant(&env);
    }

    /// Receive a loan repayment (principal + interest) from CreditLine.
    ///
    /// `principal` is tokens of principal pulled from CreditLine.
    /// `unlock` is how much of that principal was actually pool-funded and
    /// should reduce `locked_liquidity`. Any excess (`principal - unlock`) is
    /// the guarantee slice and is added to `total_liquidity` instead.
    /// `interest` is distributed via `distribute_interest`.
    pub fn receive_repayment(
        env: Env,
        creditline: Address,
        principal: i128,
        interest: i128,
        unlock: i128,
    ) {
        creditline.require_auth();
        access::require_creditline(&env, &creditline);
        Self::require_not_paused(&env);

        if principal < 0 || interest < 0 || unlock < 0 || unlock > principal {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let total = principal
            .checked_add(interest)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));

        if total <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }
        Self::enter_non_reentrant(&env);

        let locked = storage::get_locked_liquidity(&env);
        let new_locked = locked
            .checked_sub(unlock)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_locked_liquidity(&env, new_locked);

        let extra_principal = principal
            .checked_sub(unlock)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        if extra_principal > 0 {
            let total_liquidity = storage::get_total_liquidity(&env);
            let new_total = total_liquidity
                .checked_add(extra_principal)
                .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
            storage::set_total_liquidity(&env, new_total);
        }

        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&creditline, &env.current_contract_address(), &total);

        events::emit_repayment_received(&env, &creditline, principal, interest);

        if interest > 0 {
            Self::distribute_interest_internal(&env, interest);
        }
        Self::exit_non_reentrant(&env);
    }

    /// Receive a forfeited guarantee on loan default.
    /// The amount offsets the loss: it is added back to total_liquidity
    /// and reduces locked_liquidity by the same amount (partial recovery).
    pub fn receive_guarantee(env: Env, creditline: Address, amount: i128) {
        creditline.require_auth();
        access::require_creditline(&env, &creditline);
        Self::require_not_paused(&env);

        if amount <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }
        Self::enter_non_reentrant(&env);

        // The defaulted loan principal stays "locked" — the guarantee partially
        // covers the loss.  We reduce locked_liquidity by the guarantee amount
        // and add it back to total_liquidity (net pool recovers that portion).
        let locked = storage::get_locked_liquidity(&env);
        let recovered = amount.min(locked); // can't recover more than locked
        let new_locked = locked
            .checked_sub(recovered)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_locked_liquidity(&env, new_locked);

        let total_liquidity = storage::get_total_liquidity(&env);
        let new_total = total_liquidity
            .checked_add(recovered)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_liquidity(&env, new_total);

        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        // Transfer only the recovered amount — keeps accounting consistent (H-3).
        token_client.transfer(&creditline, &env.current_contract_address(), &recovered);

        events::emit_guarantee_received(&env, &creditline, recovered);
        Self::exit_non_reentrant(&env);
    }

    // -------------------------------------------------------------------------
    // Interest Distribution (SC-17 core feature)
    // -------------------------------------------------------------------------

    /// Distribute interest. Only the registered CreditLine or admin may call this.
    pub fn distribute_interest(env: Env, caller: Address, interest_amount: i128) {
        caller.require_auth();
        Self::require_not_paused(&env);
        let is_admin = storage::get_admin(&env) == caller;
        let is_creditline = storage::get_creditline(&env).map_or(false, |cl| cl == caller);
        if !is_admin && !is_creditline {
            panic_with_error!(&env, LiquidityPoolError::NotCreditLine);
        }
        Self::enter_non_reentrant(&env);
        Self::distribute_interest_internal(&env, interest_amount);
        Self::exit_non_reentrant(&env);
    }

    fn distribute_interest_internal(env: &Env, interest_amount: i128) {
        if interest_amount <= 0 {
            panic_with_error!(env, LiquidityPoolError::InvalidAmount);
        }

        // 85% stays in the pool → increases share value
        let lp_amount = interest_amount
            .checked_mul(types::LP_FEE_BPS)
            .and_then(|v| v.checked_div(types::TOTAL_BPS))
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow));

        // 10% → treasury
        let protocol_amount = interest_amount
            .checked_mul(types::PROTOCOL_FEE_BPS)
            .and_then(|v| v.checked_div(types::TOTAL_BPS))
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow));

        // 5% → merchant fund (use remainder to avoid rounding dust)
        let merchant_amount = interest_amount
            .checked_sub(lp_amount)
            .and_then(|v| v.checked_sub(protocol_amount))
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Underflow));

        let token = storage::get_token(env);
        let token_client = token::Client::new(env, &token);

        // Transfer protocol fee to treasury (if configured)
        if protocol_amount > 0 {
            if let Some(treasury) = storage::get_treasury(env) {
                token_client.transfer(&env.current_contract_address(), &treasury, &protocol_amount);
            }
            // If treasury not configured, protocol fee stays in pool (benefits LPs)
        }

        // Transfer merchant incentive to merchant fund (if configured)
        if merchant_amount > 0 {
            if let Some(merchant_fund) = storage::get_merchant_fund(env) {
                token_client.transfer(
                    &env.current_contract_address(),
                    &merchant_fund,
                    &merchant_amount,
                );
            }
            // If merchant fund not configured, fee stays in pool (benefits LPs)
        }

        // LP portion (lp_amount) stays in the pool — no transfer needed.
        // Update total_liquidity to reflect the added interest (raises share price).
        let total_liquidity = storage::get_total_liquidity(env);
        let new_total = total_liquidity
            .checked_add(lp_amount)
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow));
        storage::set_total_liquidity(env, new_total);

        events::emit_interest_distributed(
            env,
            interest_amount,
            lp_amount,
            protocol_amount,
            merchant_amount,
        );
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    pub fn get_pool_stats(env: Env) -> PoolStats {
        let total_liquidity = storage::get_total_liquidity(&env);
        let locked_liquidity = storage::get_locked_liquidity(&env);
        let available_liquidity = total_liquidity.saturating_sub(locked_liquidity);
        let total_shares = storage::get_total_shares(&env);

        // Share price in basis points: (total_liquidity × 10000) / total_shares
        let share_price = if total_shares == 0 {
            types::TOTAL_BPS // Default: 1.00 expressed as 10000 bps
        } else {
            total_liquidity
                .checked_mul(types::TOTAL_BPS)
                .and_then(|v| v.checked_div(total_shares))
                .unwrap_or(types::TOTAL_BPS)
        };

        PoolStats {
            total_liquidity,
            locked_liquidity,
            available_liquidity,
            total_shares,
            share_price,
        }
    }

    pub fn get_lp_shares(env: Env, provider: Address) -> i128 {
        storage::bump_lp_shares(&env, &provider);
        storage::get_lp_shares(&env, &provider)
    }

    /// Calculate how many tokens `shares` are worth at the current share price.
    pub fn calculate_withdrawal(env: Env, shares: i128) -> i128 {
        let total_shares = storage::get_total_shares(&env);
        if total_shares == 0 {
            return 0;
        }
        let total_liquidity = storage::get_total_liquidity(&env);
        shares
            .checked_mul(total_liquidity)
            .and_then(|v| v.checked_div(total_shares))
            .unwrap_or(0)
    }

    pub fn get_token(env: Env) -> Address {
        storage::get_token(&env)
    }

    pub fn get_treasury(env: Env) -> Option<Address> {
        storage::get_treasury(&env)
    }

    pub fn get_merchant_fund(env: Env) -> Option<Address> {
        storage::get_merchant_fund(&env)
    }

    pub fn get_creditline(env: Env) -> Option<Address> {
        storage::get_creditline(&env)
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    /// Single place where share ownership moves — every transfer path goes
    /// through it, so the share-accounting invariants hold by construction:
    /// `total_shares` is untouched and the sum of balances is preserved.
    fn move_shares(env: &Env, from: &Address, to: &Address, amount: i128) {
        let from_shares = storage::get_lp_shares(env, from);
        if from_shares < amount {
            panic_with_error!(env, LiquidityPoolError::InsufficientShares);
        }

        // A self-transfer is a no-op on balances. Skipping the writes avoids
        // reading `to` before `from` is written back, which would otherwise
        // mint `amount` shares out of thin air.
        if from != to {
            let new_from_shares = from_shares
                .checked_sub(amount)
                .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Underflow));
            let new_to_shares = storage::get_lp_shares(env, to)
                .checked_add(amount)
                .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow));

            storage::set_lp_shares(env, from, new_from_shares);
            storage::set_lp_shares(env, to, new_to_shares);
        }

        storage::bump_lp_shares(env, from);
        storage::bump_lp_shares(env, to);
    }

    /// Deduct `amount` from the allowance `from` granted `spender`, keeping the
    /// original expiration ledger.
    fn spend_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
        let allowance = storage::get_allowance(env, from, spender);
        if allowance.amount < amount {
            panic_with_error!(env, LiquidityPoolError::InsufficientAllowance);
        }

        let remaining = allowance
            .amount
            .checked_sub(amount)
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Underflow));
        storage::set_allowance(env, from, spender, remaining, allowance.expiration_ledger);
    }

    /// Rate curve parameters, falling back to the compiled-in defaults for a
    /// pool that predates the rate model (none created without `initialize`).
    fn rate_params(env: &Env) -> types::RateParams {
        storage::get_rate_params(env).unwrap_or(types::RateParams {
            base_rate_bps: types::DEFAULT_BASE_RATE_BPS,
            slope1_bps: types::DEFAULT_SLOPE1_BPS,
            slope2_bps: types::DEFAULT_SLOPE2_BPS,
            optimal_utilization_bps: types::DEFAULT_OPTIMAL_UTILIZATION_BPS,
        })
    }

    /// Utilization in bps = `locked_liquidity * 10000 / total_liquidity`, 0 when
    /// the pool is empty.
    fn utilization_bps_internal(env: &Env) -> i128 {
        let total = storage::get_total_liquidity(env);
        if total <= 0 {
            return 0;
        }
        let locked = storage::get_locked_liquidity(env);
        locked
            .checked_mul(types::TOTAL_BPS)
            .and_then(|v| v.checked_div(total))
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow))
    }

    /// The kinked rate curve, in bps, for a given utilization (clamped to
    /// 0..=10000). Below the kink the rate rises with `slope1`; above it, the
    /// portion beyond `optimal` rises with the steeper `slope2`:
    ///
    /// ```text
    /// u <= optimal: base + slope1 * u / 10000
    /// u >  optimal: base + slope1 * optimal / 10000 + slope2 * (u - optimal) / 10000
    /// ```
    fn compute_rate_bps(env: &Env, params: &types::RateParams, utilization_bps: i128) -> i128 {
        let u = utilization_bps.max(0).min(types::TOTAL_BPS);
        let optimal = params.optimal_utilization_bps;

        let scaled = |slope: i128, span: i128| -> i128 {
            slope
                .checked_mul(span)
                .and_then(|v| v.checked_div(types::TOTAL_BPS))
                .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow))
        };

        if u <= optimal {
            let inc = scaled(params.slope1_bps, u);
            params
                .base_rate_bps
                .checked_add(inc)
                .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow))
        } else {
            let inc1 = scaled(params.slope1_bps, optimal);
            let inc2 = scaled(params.slope2_bps, u - optimal);
            params
                .base_rate_bps
                .checked_add(inc1)
                .and_then(|v| v.checked_add(inc2))
                .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::Overflow))
        }
    }

    fn require_not_paused(env: &Env) {
        if Self::is_paused(env.clone()) {
            panic_with_error!(env, LiquidityPoolError::ContractPaused);
        }
    }

    fn enter_non_reentrant(env: &Env) {
        if storage::is_reentrancy_locked(env) {
            panic_with_error!(env, LiquidityPoolError::ReentrancyDetected);
        }
        storage::set_reentrancy_locked(env, true);
    }

    fn exit_non_reentrant(env: &Env) {
        storage::set_reentrancy_locked(env, false);
    }
}

#[cfg(test)]
mod tests;
