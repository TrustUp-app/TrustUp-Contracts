use soroban_sdk::{symbol_short, Address, Env, Symbol};

const ESCROW_LOCKED: Symbol = symbol_short!("ESCRWLCK");
const ESCROW_RELEASED: Symbol = symbol_short!("ESCRWRLS");
const ESCROW_SEIZED: Symbol = symbol_short!("ESCRWSZD");

/// Emitted when a guarantee is locked into escrow.
pub fn emit_locked(env: &Env, loan_id: u64, borrower: &Address, amount: i128) {
    env.events()
        .publish((ESCROW_LOCKED, loan_id), (borrower, amount));
}

/// Emitted when a guarantee is released back to the borrower.
pub fn emit_released(env: &Env, loan_id: u64, borrower: &Address, amount: i128) {
    env.events()
        .publish((ESCROW_RELEASED, loan_id), (borrower, amount));
}

/// Emitted when a guarantee is seized and forwarded to the liquidity pool.
pub fn emit_seized(env: &Env, loan_id: u64, pool: &Address, amount: i128) {
    env.events()
        .publish((ESCROW_SEIZED, loan_id), (pool, amount));
}
