#![no_std]
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
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

        Self::validate_merchant(&env, &merchant);

        Self::validate_reputation(&env, &user);

        Self::validate_liquidity(&env, total_amount, guarantee_amount);

        let loan_id = storage::increment_loan_counter(&env);

        // Create loan record
        let loan = Loan {
            loan_id,
            borrower: user.clone(),
            merchant: merchant.clone(),
            total_amount,
            guarantee_amount,
            remaining_balance: total_amount,
            repayment_schedule: repayment_schedule.clone(),
            status: LoanStatus::Active,
            created_at: env.ledger().timestamp(),
        };

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
    /// TODO: Implement when Merchant Registry contract is available (Phase 5)
    fn validate_merchant(env: &Env, merchant: &Address) {
        let merchant_registry = storage::get_merchant_registry(env);

        if merchant_registry.is_none() {
            // Merchant registry not configured yet
            // For now, we'll skip this validation
            // TODO: Remove this when merchant registry is implemented
            return;
        }

        // TODO: Query merchant registry contract
        // Example: merchant_registry_client.is_active(merchant)
        // For now, we assume merchant is valid
        let _ = merchant;
    }

    /// Validate user has sufficient reputation
    fn validate_reputation(env: &Env, user: &Address) {
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
    }

    /// Validate liquidity pool has sufficient funds
    /// TODO: Implement when Liquidity Pool contract is available (Phase 6)
    fn validate_liquidity(env: &Env, total_amount: i128, guarantee_amount: i128) {
        let liquidity_pool = storage::get_liquidity_pool(env);

        if liquidity_pool.is_none() {
            // Liquidity pool not configured yet
            // For now, we'll skip this validation
            // TODO: Remove this when liquidity pool is implemented
            return;
        }

        // The loan requires (total_amount - guarantee_amount) from the pool
        let required_from_pool = total_amount
            .checked_sub(guarantee_amount)
            .unwrap_or_else(|| panic_with_error!(env, CreditLineError::Underflow));

        // TODO: Query liquidity pool contract
        // Example: liquidity_pool_client.get_available_liquidity()
        // For now, we assume liquidity is sufficient
        let _ = required_from_pool;
    }
}

#[cfg(test)]
mod tests;
