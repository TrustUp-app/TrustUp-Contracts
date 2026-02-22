#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Env};

mod errors;
mod events;
mod storage;
mod types;
mod liquidity_pool_tests;

pub use errors::LiquidityPoolError;
pub use types::PoolStats;

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
    }

    // -------------------------------------------------------------------------
    // Admin setters
    // -------------------------------------------------------------------------

    pub fn set_creditline(env: Env, admin: Address, creditline: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        storage::set_creditline(&env, &creditline);
    }

    pub fn set_treasury(env: Env, admin: Address, treasury: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        storage::set_treasury(&env, &treasury);
    }

    pub fn set_merchant_fund(env: Env, admin: Address, merchant_fund: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        storage::set_merchant_fund(&env, &merchant_fund);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let old_admin = storage::get_admin(&env);
        old_admin.require_auth();
        Self::require_admin(&env, &old_admin);
        storage::set_admin(&env, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
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

        if amount <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

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

        // Transfer tokens from provider to pool contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&provider, &env.current_contract_address(), &amount);

        // Update state
        let new_shares = storage::get_lp_shares(&env, &provider)
            .checked_add(shares_issued)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_lp_shares(&env, &provider, new_shares);

        let new_total_shares = total_shares
            .checked_add(shares_issued)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_shares(&env, new_total_shares);

        let new_total_liquidity = total_liquidity
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_liquidity(&env, new_total_liquidity);

        events::emit_liquidity_deposited(&env, &provider, amount, shares_issued);

        shares_issued
    }

    /// Burn `shares` and return the proportional token amount to `provider`.
    ///
    /// `amount = (shares × total_pool_value) / total_shares`
    ///
    /// Returns the number of tokens returned.
    pub fn withdraw(env: Env, provider: Address, shares: i128) -> i128 {
        provider.require_auth();

        if shares <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

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

        let new_total_shares = total_shares
            .checked_sub(shares)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_total_shares(&env, new_total_shares);

        let new_total_liquidity = total_liquidity
            .checked_sub(amount_returned)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_total_liquidity(&env, new_total_liquidity);

        // Transfer tokens back to provider
        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &provider, &amount_returned);

        events::emit_liquidity_withdrawn(&env, &provider, shares, amount_returned);

        amount_returned
    }

    // -------------------------------------------------------------------------
    // CreditLine Operations (access-restricted)
    // -------------------------------------------------------------------------

    /// Transfer `amount` tokens to `merchant` to fund a loan.
    /// Only the registered CreditLine contract may call this.
    pub fn fund_loan(env: Env, creditline: Address, merchant: Address, amount: i128) {
        creditline.require_auth();
        Self::require_creditline(&env, &creditline);

        if amount <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let total_liquidity = storage::get_total_liquidity(&env);
        let locked_liquidity = storage::get_locked_liquidity(&env);
        let available = total_liquidity
            .checked_sub(locked_liquidity)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));

        if amount > available {
            panic_with_error!(&env, LiquidityPoolError::InsufficientLiquidity);
        }

        // Transfer tokens from pool to merchant
        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &merchant, &amount);

        let new_locked = locked_liquidity
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_locked_liquidity(&env, new_locked);

        events::emit_loan_funded(&env, &creditline, amount);
    }

    /// Receive a loan repayment (principal + interest) from CreditLine.
    ///
    /// `principal` reduces locked_liquidity (loan is repaid).
    /// `interest`  is distributed via `distribute_interest` (increases pool value).
    pub fn receive_repayment(
        env: Env,
        creditline: Address,
        principal: i128,
        interest: i128,
    ) {
        creditline.require_auth();
        Self::require_creditline(&env, &creditline);

        if principal < 0 || interest < 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let total = principal
            .checked_add(interest)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));

        if total <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        // Accept funds from CreditLine
        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&creditline, &env.current_contract_address(), &total);

        // Decrease locked liquidity by the principal
        let locked = storage::get_locked_liquidity(&env);
        let new_locked = locked
            .checked_sub(principal)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Underflow));
        storage::set_locked_liquidity(&env, new_locked);

        // Principal returns to total_liquidity, then distribute interest
        let total_liquidity = storage::get_total_liquidity(&env);
        let after_principal = total_liquidity
            .checked_add(principal)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_liquidity(&env, after_principal);

        events::emit_repayment_received(&env, &creditline, principal, interest);

        if interest > 0 {
            Self::distribute_interest(&env, interest);
        }
    }

    /// Receive a forfeited guarantee on loan default.
    /// The amount offsets the loss: it is added back to total_liquidity
    /// and reduces locked_liquidity by the same amount (partial recovery).
    pub fn receive_guarantee(env: Env, creditline: Address, amount: i128) {
        creditline.require_auth();
        Self::require_creditline(&env, &creditline);

        if amount <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let token = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&creditline, &env.current_contract_address(), &amount);

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

        events::emit_guarantee_received(&env, &creditline, amount);
    }

    // -------------------------------------------------------------------------
    // Interest Distribution (SC-17 core feature)
    // -------------------------------------------------------------------------

    /// Distribute `interest_amount` according to the protocol fee split:
    ///   - 85 % → Liquidity Providers  (increases share value by raising `total_liquidity`)
    ///   - 10 % → Protocol Treasury
    ///   -  5 % → Merchant Incentive Fund
    ///
    /// The LP portion is NOT transferred out; it stays in the pool and inflates
    /// the share price (existing LP shares become worth more).
    ///
    /// This function is called internally by `receive_repayment`, but it is also
    /// `pub` so that the CreditLine (or admin, in edge-case) can call it directly.
    pub fn distribute_interest(env: &Env, interest_amount: i128) {
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
                token_client.transfer(
                    &env.current_contract_address(),
                    &treasury,
                    &protocol_amount,
                );
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

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn require_admin(env: &Env, caller: &Address) {
        let admin = storage::get_admin(env);
        if admin != *caller {
            panic_with_error!(env, LiquidityPoolError::NotAdmin);
        }
    }

    fn require_creditline(env: &Env, caller: &Address) {
        let creditline = storage::get_creditline(env)
            .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::NotCreditLine));
        if creditline != *caller {
            panic_with_error!(env, LiquidityPoolError::NotCreditLine);
        }
    }
}

#[cfg(test)]
mod tests;
