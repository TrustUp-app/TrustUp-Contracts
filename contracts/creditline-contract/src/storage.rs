use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

use crate::types::Loan;

// Storage keys
pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const LOAN_COUNTER: Symbol = symbol_short!("LOANCNT");
pub const REPUTATION_CONTRACT: Symbol = symbol_short!("REPCONT");
pub const MERCHANT_REGISTRY: Symbol = symbol_short!("MERCHANT");
pub const LIQUIDITY_POOL: Symbol = symbol_short!("LIQPOOL");
pub const TOKEN: Symbol = symbol_short!("TOKEN");

const LOAN_SHARD_COUNT: u32 = 32;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Loan(u32, u64),
    UserLoanCount(Address),
    UserLoanAt(Address, u64),
    UserActiveDebt(Address),
}

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
    let shard = loan_shard(loan_id);
    env.storage()
        .persistent()
        .get(&DataKey::Loan(shard, loan_id))
}

/// Write a loan to storage
pub fn write_loan(env: &Env, loan: &Loan) {
    let shard = loan_shard(loan.loan_id);
    let key = DataKey::Loan(shard, loan.loan_id);
    let is_new = !env.storage().persistent().has(&key);
    env.storage().persistent().set(&key, loan);

    if is_new {
        append_user_loan_index(env, &loan.borrower, loan.loan_id);
    }
}

pub fn get_user_loan_count(env: &Env, borrower: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::UserLoanCount(borrower.clone()))
        .unwrap_or(0)
}

pub fn get_user_loan_ids_paginated(
    env: &Env,
    borrower: &Address,
    start: u64,
    limit: u32,
) -> Vec<u64> {
    let total = get_user_loan_count(env, borrower);
    let mut result = Vec::new(env);

    if limit == 0 || start >= total {
        return result;
    }

    let end = start.saturating_add(limit as u64).min(total);
    let mut idx = start;
    while idx < end {
        let key = DataKey::UserLoanAt(borrower.clone(), idx);
        if let Some(loan_id) = env.storage().persistent().get::<DataKey, u64>(&key) {
            result.push_back(loan_id);
        }
        idx += 1;
    }

    result
}

pub fn get_user_loans_paginated(
    env: &Env,
    borrower: &Address,
    start: u64,
    limit: u32,
) -> Vec<Loan> {
    let loan_ids = get_user_loan_ids_paginated(env, borrower, start, limit);
    let mut loans = Vec::new(env);

    for loan_id in loan_ids.iter() {
        if let Some(loan) = read_loan(env, loan_id) {
            loans.push_back(loan);
        }
    }

    loans
}

pub fn get_user_active_debt(env: &Env, borrower: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::UserActiveDebt(borrower.clone()))
        .unwrap_or(0)
}

pub fn increase_user_active_debt(env: &Env, borrower: &Address, amount: i128) {
    let current = get_user_active_debt(env, borrower);
    let next = current
        .checked_add(amount)
        .expect("User active debt overflow");
    env.storage()
        .persistent()
        .set(&DataKey::UserActiveDebt(borrower.clone()), &next);
}

pub fn decrease_user_active_debt(env: &Env, borrower: &Address, amount: i128) {
    let current = get_user_active_debt(env, borrower);
    let next = current
        .checked_sub(amount)
        .expect("User active debt underflow");
    env.storage()
        .persistent()
        .set(&DataKey::UserActiveDebt(borrower.clone()), &next);
}

fn append_user_loan_index(env: &Env, borrower: &Address, loan_id: u64) {
    let count = get_user_loan_count(env, borrower);
    env.storage()
        .persistent()
        .set(&DataKey::UserLoanAt(borrower.clone(), count), &loan_id);
    env.storage()
        .persistent()
        .set(&DataKey::UserLoanCount(borrower.clone()), &(count + 1));
}

fn loan_shard(loan_id: u64) -> u32 {
    (loan_id % (LOAN_SHARD_COUNT as u64)) as u32
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

/// Get the Token Contract address
pub fn get_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&TOKEN)
}

/// Set the Token Contract address
pub fn set_token(env: &Env, address: &Address) {
    env.storage().instance().set(&TOKEN, address);
}
