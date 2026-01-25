use soroban_sdk::{symbol_short, Address, Env, Map, Symbol};

use crate::types::Loan;

// Storage keys
pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const LOAN_COUNTER: Symbol = symbol_short!("LOANCNT");
pub const LOANS_MAP: Symbol = symbol_short!("LOANS");
pub const REPUTATION_CONTRACT: Symbol = symbol_short!("REPCONT");
pub const MERCHANT_REGISTRY: Symbol = symbol_short!("MERCHANT");
pub const LIQUIDITY_POOL: Symbol = symbol_short!("LIQPOOL");

/// Get the admin address from storage
pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .unwrap_or_else(|| panic!("Admin not set"))
}

/// Set the admin address in storage
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN_KEY, admin);
}

/// Get the current loan counter (for generating unique loan IDs)
pub fn get_loan_counter(env: &Env) -> u64 {
    env.storage().instance().get(&LOAN_COUNTER).unwrap_or(0)
}

/// Increment and return the next loan ID
pub fn increment_loan_counter(env: &Env) -> u64 {
    let current = get_loan_counter(env);
    let next = current.checked_add(1).expect("Loan counter overflow");
    env.storage().instance().set(&LOAN_COUNTER, &next);
    next
}

/// Read a loan from storage
pub fn read_loan(env: &Env, loan_id: u64) -> Option<Loan> {
    let loans: Map<u64, Loan> = env
        .storage()
        .instance()
        .get(&LOANS_MAP)
        .unwrap_or_else(|| Map::new(env));

    loans.get(loan_id)
}

/// Write a loan to storage
pub fn write_loan(env: &Env, loan: &Loan) {
    let mut loans: Map<u64, Loan> = env
        .storage()
        .instance()
        .get(&LOANS_MAP)
        .unwrap_or_else(|| Map::new(env));

    loans.set(loan.loan_id, loan.clone());
    env.storage().instance().set(&LOANS_MAP, &loans);
}

/// Get the Reputation Contract address
pub fn get_reputation_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&REPUTATION_CONTRACT)
}

/// Set the Reputation Contract address
pub fn set_reputation_contract(env: &Env, address: &Address) {
    env.storage().instance().set(&REPUTATION_CONTRACT, address);
}

/// Get the Merchant Registry Contract address
pub fn get_merchant_registry(env: &Env) -> Option<Address> {
    env.storage().instance().get(&MERCHANT_REGISTRY)
}

/// Set the Merchant Registry Contract address
pub fn set_merchant_registry(env: &Env, address: &Address) {
    env.storage().instance().set(&MERCHANT_REGISTRY, address);
}

/// Get the Liquidity Pool Contract address
pub fn get_liquidity_pool(env: &Env) -> Option<Address> {
    env.storage().instance().get(&LIQUIDITY_POOL)
}

/// Set the Liquidity Pool Contract address
pub fn set_liquidity_pool(env: &Env, address: &Address) {
    env.storage().instance().set(&LIQUIDITY_POOL, address);
}
