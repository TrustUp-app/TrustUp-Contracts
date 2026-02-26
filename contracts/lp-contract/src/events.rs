use soroban_sdk::{symbol_short, Address, Env, Symbol};

const DEPOSITED: Symbol = symbol_short!("LQDEPST");

/// Emitted when a liquidity provider deposits tokens.
/// Topics : (LQDEPST, provider)
/// Data   : (amount, shares_issued)
pub fn emit_liquidity_deposited(env: &Env, provider: &Address, amount: i128, shares_issued: i128) {
    env.events()
        .publish((DEPOSITED, provider), (amount, shares_issued));
}
