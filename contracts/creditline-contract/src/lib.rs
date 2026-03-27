#![no_std]
use liquidity_pool_contract::LiquidityPoolContractClient;
use merchant_registry_contract::MerchantRegistryContractClient;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, Address, Env, IntoVal, Symbol,
    Vec,
};

// Module imports
mod access;
mod errors;
mod events;
mod storage;
mod types;

// Re-export types for external use
pub use errors::CreditLineError;
pub use types::{Loan, LoanStatus, RepaymentInstallment};

/// CreditLine contract structure
#[contract]
pub struct CreditLineContract;

/// Contract implementation
#[contractimpl]
impl CreditLineContract {
    /// Get the version of this contract
    pub fn get_version() -> Symbol {
        symbol_short!("v1_0_0")
    }

    /// Initialize the contract with admin and external contract addresses
    /// Can only be called once (when admin is not set)
    pub fn initialize(
        env: Env,
        admin: Address,
        reputation_contract: Address,
        merchant_registry: Address,
        liquidity_pool: Address,
        token: Address,
    ) {
        // Check if already initialized
        let admin_opt: Option<Address> = env.storage().instance().get(&storage::ADMIN_KEY);
        if admin_opt.is_some() {
            panic!("Already initialized");
        }

        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_reputation_contract(&env, &reputation_contract);
        storage::set_merchant_registry(&env, &merchant_registry);
        storage::set_liquidity_pool(&env, &liquidity_pool);
        storage::set_token(&env, &token);
    }

    /// Create a new loan
    /// Validates all requirements and creates an active loan
    pub fn create_loan(
        env: Env,
        user: Address,
        merchant: Address,
        total_amount: i128,
        guarantee_amount: i128,
        repayment_schedule: Vec<RepaymentInstallment>,
    ) -> u64 {
        user.require_auth();

        Self::validate_guarantee(&env, total_amount, guarantee_amount);
        let score = Self::validate_reputation(&env, &user);
        let mut loan = Self::build_loan(
            &env,
            user.clone(),
            merchant.clone(),
            total_amount,
            guarantee_amount,
            repayment_schedule.clone(),
            score,
            LoanStatus::Active,
        );

        Self::validate_liquidity(&env, total_amount, guarantee_amount);
        Self::fund_loan_from_pool(&env, &user, &merchant, guarantee_amount, total_amount);
        loan.funded_at = env.ledger().timestamp();

        storage::increase_user_active_debt(&env, &user, loan.remaining_balance);
        let loan_id = loan.loan_id;

        storage::write_loan(&env, &loan);

        events::emit_loan_created(
            &env,
            &user,
            &merchant,
            loan_id,
            total_amount,
            guarantee_amount,
            &repayment_schedule,
        );

        loan_id
    }

    /// Create an unfunded loan request. Borrower guarantee is escrowed until the
    /// request is either funded by a future workflow or explicitly cancelled.
    pub fn request_loan(
        env: Env,
        user: Address,
        merchant: Address,
        total_amount: i128,
        guarantee_amount: i128,
        repayment_schedule: Vec<RepaymentInstallment>,
    ) -> u64 {
        user.require_auth();

        Self::validate_guarantee(&env, total_amount, guarantee_amount);
        let score = Self::validate_reputation(&env, &user);
        let loan = Self::build_loan(
            &env,
            user.clone(),
            merchant.clone(),
            total_amount,
            guarantee_amount,
            repayment_schedule.clone(),
            score,
            LoanStatus::Pending,
        );

        let token_address = storage::get_token(&env)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::TokenNotConfigured));
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&user, &env.current_contract_address(), &guarantee_amount);

        let loan_id = loan.loan_id;
        storage::write_loan(&env, &loan);

        events::emit_loan_requested(
            &env,
            &user,
            &merchant,
            loan_id,
            total_amount,
            guarantee_amount,
            &repayment_schedule,
        );

        loan_id
    }

    /// Paginated borrower loan history for scalable reads.
    pub fn get_user_loans(env: Env, borrower: Address, start: u64, limit: u32) -> Vec<Loan> {
        storage::get_user_loans_paginated(&env, &borrower, start, limit)
    }

    pub fn get_user_loan_count(env: Env, borrower: Address) -> u64 {
        storage::get_user_loan_count(&env, &borrower)
    }

    pub fn get_user_active_debt(env: Env, borrower: Address) -> i128 {
        storage::get_user_active_debt(&env, &borrower)
    }

    /// Get a loan by ID
    pub fn get_loan(env: Env, loan_id: u64) -> Loan {
        storage::read_loan(&env, loan_id)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::LoanNotFound))
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let old_admin = storage::get_admin(&env);
        old_admin.require_auth();
        access::require_admin(&env, &old_admin);

        storage::set_admin(&env, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    /// Set the reputation contract address (admin only)
    pub fn set_reputation_contract(env: Env, admin: Address, address: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        storage::set_reputation_contract(&env, &address);
    }

    /// Set the merchant registry contract address (admin only)
    pub fn set_merchant_registry(env: Env, admin: Address, address: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        storage::set_merchant_registry(&env, &address);
    }

    /// Set the liquidity pool contract address (admin only)
    pub fn set_liquidity_pool(env: Env, admin: Address, address: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        storage::set_liquidity_pool(&env, &address);
    }

    /// Validate guarantee amount is at least 20% of total amount
    fn validate_guarantee(env: &Env, total_amount: i128, guarantee_amount: i128) {
        if total_amount <= 0 || guarantee_amount <= 0 {
            panic_with_error!(env, CreditLineError::InvalidAmount);
        }

        if guarantee_amount > total_amount {
            panic_with_error!(env, CreditLineError::InvalidAmount);
        }

        // Calculate minimum guarantee (20% of total)
        let min_guarantee = total_amount
            .checked_mul(types::MIN_GUARANTEE_PERCENT)
            .and_then(|v| v.checked_div(100))
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::Overflow));

        if guarantee_amount < min_guarantee {
            panic_with_error!(env, CreditLineError::InsufficientGuarantee);
        }
    }

    /// Validate merchant is registered and active
    fn validate_merchant(env: &Env, merchant: &Address) {
        let merchant_registry = storage::get_merchant_registry(env)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::InvalidMerchant));

        let registry_client = MerchantRegistryContractClient::new(env, &merchant_registry);
        let is_active = env
            .try_invoke_contract::<bool, soroban_sdk::Error>(
                &registry_client.address,
                &symbol_short!("is_active"),
                (merchant,).into_val(env),
            )
            .unwrap_or_else(|_| panic_with_error!(env, CreditLineError::MerchantValidationFailed))
            .unwrap_or_else(|_| panic_with_error!(env, CreditLineError::MerchantValidationFailed));

        if !is_active {
            panic_with_error!(env, CreditLineError::MerchantNotActive);
        }
    }

    /// Validate user has sufficient reputation
    fn validate_reputation(env: &Env, user: &Address) -> u32 {
        let reputation_contract = storage::get_reputation_contract(env)
            .unwrap_or_else(|| panic!("Reputation contract not configured"));

        // Call the reputation contract to get user's score
        // Using the reputation contract interface
        use soroban_sdk::IntoVal;

        let score: u32 = env.invoke_contract(
            &reputation_contract,
            &symbol_short!("get_score"),
            (user,).into_val(env),
        );

        if score < types::MIN_REPUTATION_THRESHOLD {
            panic_with_error!(env, CreditLineError::InsufficientReputation);
        }

        score
    }

    fn validate_liquidity(env: &Env, total_amount: i128, guarantee_amount: i128) {
        let liquidity_pool = storage::get_liquidity_pool(env)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::InsufficientLiquidity));

        let lp_client = LiquidityPoolContractClient::new(env, &liquidity_pool);
        let stats = lp_client.get_pool_stats();

        if stats.available_liquidity < total_amount {
            panic_with_error!(env, CreditLineError::InsufficientLiquidity);
        }

        let required_guarantee = total_amount
            .checked_sub(guarantee_amount)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::Underflow));
        if required_guarantee < 0 {
            panic_with_error!(env, CreditLineError::InvalidAmount);
        }
    }

    fn fund_loan_from_pool(
        env: &Env,
        borrower: &Address,
        merchant: &Address,
        guarantee_amount: i128,
        principal_amount: i128,
    ) {
        let liquidity_pool = storage::get_liquidity_pool(env)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::InsufficientLiquidity));

        let token_address = storage::get_token(env)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::TokenNotConfigured));

        let token_client = token::Client::new(env, &token_address);
        // Escrow borrower guarantee in CreditLine. It is forwarded to the pool only on default.
        token_client.transfer(borrower, &env.current_contract_address(), &guarantee_amount);

        let lp_client = LiquidityPoolContractClient::new(env, &liquidity_pool);
        lp_client.fund_loan(&env.current_contract_address(), merchant, &principal_amount);
    }

    fn build_loan(
        env: &Env,
        user: Address,
        merchant: Address,
        total_amount: i128,
        guarantee_amount: i128,
        repayment_schedule: Vec<RepaymentInstallment>,
        score: u32,
        status: LoanStatus,
    ) -> Loan {
        Self::validate_guarantee(env, total_amount, guarantee_amount);
        Self::validate_merchant(env, &merchant);

        let interest_rate_bps = Self::interest_rate_bps(score);
        let interest_amount =
            Self::calculate_bps_amount(env, total_amount, interest_rate_bps as i128);
        let service_fee_amount =
            Self::calculate_bps_amount(env, total_amount, types::SERVICE_FEE_BPS);
        let remaining_balance = total_amount
            .checked_add(interest_amount)
            .and_then(|v| v.checked_add(service_fee_amount))
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::Overflow));

        let credit_limit = Self::credit_limit(score);
        let active_debt = storage::get_user_active_debt(env, &user);
        let next_debt = active_debt
            .checked_add(remaining_balance)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::Overflow));
        if next_debt > credit_limit {
            panic_with_error!(env, CreditLineError::ExposureLimitExceeded);
        }

        let loan_id = storage::increment_loan_counter(env);
        Loan {
            loan_id,
            borrower: user,
            merchant,
            total_amount,
            guarantee_amount,
            interest_rate_bps,
            interest_amount,
            service_fee_amount,
            principal_outstanding: total_amount,
            interest_outstanding: interest_amount,
            service_fee_outstanding: service_fee_amount,
            remaining_balance,
            repayment_schedule,
            status,
            created_at: env.ledger().timestamp(),
            funded_at: 0,
        }
    }

    fn calculate_bps_amount(env: &Env, base: i128, bps: i128) -> i128 {
        base.checked_mul(bps)
            .and_then(|v| v.checked_div(types::BPS_DENOMINATOR))
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::Overflow))
    }

    fn interest_rate_bps(score: u32) -> u32 {
        match score {
            90..=u32::MAX => 400,
            75..=89 => 600,
            60..=74 => 800,
            _ => 1_000,
        }
    }

    fn credit_limit(score: u32) -> i128 {
        match score {
            90..=u32::MAX => 10_000,
            75..=89 => 5_000,
            60..=74 => 2_500,
            _ => 1_000,
        }
    }

    /// Calculate appropriate penalty amount (20-30 points based on loan size)
    fn calculate_default_penalty(loan: &Loan) -> u32 {
        // Simple logic: 20 points base penalty, 30 points if loan > 5000 units
        if loan.total_amount > 5000 {
            30
        } else {
            20
        }
    }

    pub fn mark_defaulted(env: Env, loan_id: u64) -> Result<(), CreditLineError> {
        // 1. Validation: Loan must exist
        let mut loan = storage::read_loan(&env, loan_id).ok_or(CreditLineError::LoanNotFound)?;

        // 2. Validation: Loan must be Active
        if loan.status != LoanStatus::Active {
            return Err(CreditLineError::LoanNotActive);
        }

        // 3. Validation: Check if past final payment date
        // We look at the last installment in the schedule
        let last_installment = loan
            .repayment_schedule
            .last()
            .ok_or(CreditLineError::Overflow)?; // Should never happen with valid loans

        if env.ledger().timestamp() <= last_installment.due_date {
            return Err(CreditLineError::LoanNotOverdue);
        }

        // 4. Transfer guarantee to Liquidity Pool
        let lp_address =
            storage::get_liquidity_pool(&env).ok_or(CreditLineError::InsufficientLiquidity)?;
        let token_address = storage::get_token(&env).ok_or(CreditLineError::TokenNotConfigured)?;

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &lp_address,
            &loan.guarantee_amount,
        );

        let lp_client = LiquidityPoolContractClient::new(&env, &lp_address);
        lp_client.receive_guarantee(&env.current_contract_address(), &loan.guarantee_amount);

        // 5. Update Status
        storage::decrease_user_active_debt(&env, &loan.borrower, loan.remaining_balance);
        loan.status = LoanStatus::Defaulted;
        storage::write_loan(&env, &loan);

        // 6. Emit Event
        events::emit_loan_defaulted(
            &env,
            loan.borrower.clone(),
            loan_id,
            loan.total_amount,
            loan.remaining_balance,
            loan.guarantee_amount,
        );

        // 7. Trigger reputation decrease
        if let Some(reputation_contract) = storage::get_reputation_contract(&env) {
            let penalty = Self::calculate_default_penalty(&loan);
            let updater = env.current_contract_address();

            // Call decrease_score(updater, user, amount)
            // Error handling: if the reputation call fails, we still want the loan to be marked as defaulted.
            // Using try_invoke_contract allows us to catch the failure and log it without rolling back the whole transaction.
            let _ = env.try_invoke_contract::<(), soroban_sdk::Error>(
                &reputation_contract,
                &Symbol::new(&env, "decrease_score"),
                (updater, loan.borrower, penalty).into_val(&env),
            );
        }

        Ok(())
    }

    pub fn cancel_loan(env: Env, caller: Address, loan_id: u64) {
        caller.require_auth();

        let mut loan = storage::read_loan(&env, loan_id)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::LoanNotFound));

        if loan.status != LoanStatus::Pending {
            panic_with_error!(&env, CreditLineError::LoanNotCancellable);
        }

        let admin = storage::get_admin(&env);
        if caller != loan.borrower && caller != admin {
            panic_with_error!(&env, CreditLineError::UnauthorizedRepayer);
        }

        let token_address = storage::get_token(&env)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::TokenNotConfigured));
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &loan.borrower,
            &loan.guarantee_amount,
        );

        loan.status = LoanStatus::Cancelled;
        storage::write_loan(&env, &loan);
        events::emit_loan_cancelled(&env, &loan.borrower, loan_id, loan.guarantee_amount);
    }

    /// Repay a loan (partial or full)
    /// Returns the remaining balance after payment
    pub fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) -> i128 {
        // 1. Auth first
        borrower.require_auth();

        // 2. Load loan
        let mut loan = storage::read_loan(&env, loan_id)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::LoanNotFound));

        // 3. Verify borrower matches
        if loan.borrower != borrower {
            panic_with_error!(&env, CreditLineError::UnauthorizedRepayer);
        }

        // 4. Loan must be Active
        if loan.status != LoanStatus::Active {
            panic_with_error!(&env, CreditLineError::LoanNotActive);
        }

        // 5. Amount must be > 0 and <= remaining_balance
        if amount <= 0 || amount > loan.remaining_balance {
            panic_with_error!(&env, CreditLineError::InvalidRepaymentAmount);
        }

        // 6. Apply repayment principal-first to reduce protocol exposure earliest.
        let principal_paid = amount.min(loan.principal_outstanding);
        let after_principal = amount
            .checked_sub(principal_paid)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Underflow));
        let interest_paid = after_principal.min(loan.interest_outstanding);
        let after_interest = after_principal
            .checked_sub(interest_paid)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Underflow));
        let fee_paid = after_interest.min(loan.service_fee_outstanding);

        loan.principal_outstanding = loan
            .principal_outstanding
            .checked_sub(principal_paid)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Underflow));
        loan.interest_outstanding = loan
            .interest_outstanding
            .checked_sub(interest_paid)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Underflow));
        loan.service_fee_outstanding = loan
            .service_fee_outstanding
            .checked_sub(fee_paid)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Underflow));

        // 7. Calculate new remaining balance
        let new_balance = loan
            .remaining_balance
            .checked_sub(amount)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Underflow));

        // 8. Prepare updated loan state
        loan.remaining_balance = new_balance;

        let is_fully_repaid = new_balance == 0;
        if is_fully_repaid {
            loan.status = LoanStatus::Paid;
        }

        // 9. Resolve external addresses before touching anything
        let lp_address = storage::get_liquidity_pool(&env)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::InsufficientLiquidity));

        let token_address = storage::get_token(&env)
            .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::TokenNotConfigured));

        // 10. Collect the repayment in CreditLine so the pool can pull it atomically.
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&borrower, &env.current_contract_address(), &amount);

        // 11. Notify pool — hard call so pool accounting stays in sync.
        let lp_client = LiquidityPoolContractClient::new(&env, &lp_address);
        lp_client.receive_repayment(
            &env.current_contract_address(),
            &principal_paid,
            &interest_paid
                .checked_add(fee_paid)
                .unwrap_or_else(|| panic_with_error!(&env, CreditLineError::Overflow)),
        );

        if is_fully_repaid {
            token_client.transfer(
                &env.current_contract_address(),
                &borrower,
                &loan.guarantee_amount,
            );
        }

        // 12. All external calls succeeded — now safe to commit state
        storage::decrease_user_active_debt(&env, &borrower, amount);
        storage::write_loan(&env, &loan);

        // 13. Emit event
        events::emit_loan_repaid(
            &env,
            &borrower,
            loan_id,
            amount,
            new_balance,
            is_fully_repaid,
        );

        // 14. Reputation increase on full repayment — soft side-effect, failure is acceptable.
        if is_fully_repaid {
            if let Some(reputation_contract) = storage::get_reputation_contract(&env) {
                let updater = env.current_contract_address();
                let score_bonus = loan
                    .repayment_schedule
                    .first()
                    .map(|installment| {
                        if env.ledger().timestamp() < installment.due_date {
                            15u32
                        } else {
                            10u32
                        }
                    })
                    .unwrap_or(10u32);
                let _ = env.try_invoke_contract::<(), soroban_sdk::Error>(
                    &reputation_contract,
                    &Symbol::new(&env, "increase_score"),
                    (updater, borrower, score_bonus).into_val(&env),
                );
            }
        }

        new_balance
    }
}

#[cfg(test)]
mod tests;
