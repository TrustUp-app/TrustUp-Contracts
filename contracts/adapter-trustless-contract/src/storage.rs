use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::EscrowEntry;

pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const CREDITLINE_KEY: Symbol = symbol_short!("CREDITLN");
pub const TOKEN_KEY: Symbol = symbol_short!("TOKEN");
pub const ESCROW_PREFIX: Symbol = symbol_short!("ESCROW");

// --- Admin ---

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&ADMIN_KEY).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN_KEY, admin);
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&ADMIN_KEY)
}

// --- CreditLine ---

pub fn get_creditline(env: &Env) -> Address {
    env.storage().instance().get(&CREDITLINE_KEY).unwrap()
}

pub fn set_creditline(env: &Env, creditline: &Address) {
    env.storage().instance().set(&CREDITLINE_KEY, creditline);
}

// --- Token ---

pub fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&TOKEN_KEY).unwrap()
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&TOKEN_KEY, token);
}

// --- Escrow entries ---

pub fn get_escrow(env: &Env, loan_id: u64) -> Option<EscrowEntry> {
    env.storage().persistent().get(&(ESCROW_PREFIX, loan_id))
}

pub fn set_escrow(env: &Env, loan_id: u64, entry: &EscrowEntry) {
    env.storage()
        .persistent()
        .set(&(ESCROW_PREFIX, loan_id), entry);
}
