use soroban_sdk::{symbol_short, Address, Env, Symbol};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const TOKEN_KEY: Symbol = symbol_short!("TOKEN");
const TOTAL_SHARES_KEY: Symbol = symbol_short!("TOTSHRS");
const TOTAL_LIQUIDITY_KEY: Symbol = symbol_short!("TOTLIQ");
const LP_SHARES_PREFIX: Symbol = symbol_short!("LPSHRS");

// --- Admin ---

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&ADMIN_KEY)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN_KEY, admin);
}

// --- Token ---

pub fn get_token(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&TOKEN_KEY)
        .expect("Not initialized")
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&TOKEN_KEY, token);
}

// --- Total Shares ---

pub fn get_total_shares(env: &Env) -> i128 {
    env.storage().instance().get(&TOTAL_SHARES_KEY).unwrap_or(0)
}

pub fn set_total_shares(env: &Env, total: i128) {
    env.storage().instance().set(&TOTAL_SHARES_KEY, &total);
}

// --- Total Liquidity ---

pub fn get_total_liquidity(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&TOTAL_LIQUIDITY_KEY)
        .unwrap_or(0)
}

pub fn set_total_liquidity(env: &Env, total: i128) {
    env.storage().instance().set(&TOTAL_LIQUIDITY_KEY, &total);
}

// --- LP Shares (persistent per-provider) ---

pub fn get_lp_shares(env: &Env, provider: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&(LP_SHARES_PREFIX, provider.clone()))
        .unwrap_or(0)
}

pub fn set_lp_shares(env: &Env, provider: &Address, shares: i128) {
    env.storage()
        .persistent()
        .set(&(LP_SHARES_PREFIX, provider.clone()), &shares);
}
