use soroban_sdk::{symbol_short, Address, Env, Symbol};

// Instance storage keys
pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const TOKEN_KEY: Symbol = symbol_short!("TOKEN");
pub const TOTAL_SHARES_KEY: Symbol = symbol_short!("TOTSHRS");
pub const TOTAL_LIQUIDITY_KEY: Symbol = symbol_short!("TOTLIQ");
pub const LOCKED_LIQUIDITY_KEY: Symbol = symbol_short!("LCKDLIQ");
pub const CREDITLINE_KEY: Symbol = symbol_short!("CRDTLIN");
pub const TREASURY_KEY: Symbol = symbol_short!("TREASURY");
pub const MERCHANT_FUND_KEY: Symbol = symbol_short!("MRCHFND");

// Persistent storage key prefix for LP shares
pub const LP_SHARES_PREFIX: Symbol = symbol_short!("LPSHRS");

// --- Admin ---

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .expect("Not initialized")
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN_KEY, admin);
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&ADMIN_KEY)
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

// --- CreditLine ---

pub fn get_creditline(env: &Env) -> Option<Address> {
    env.storage().instance().get(&CREDITLINE_KEY)
}

pub fn set_creditline(env: &Env, creditline: &Address) {
    env.storage().instance().set(&CREDITLINE_KEY, creditline);
}

// --- Protocol Treasury ---

pub fn get_treasury(env: &Env) -> Option<Address> {
    env.storage().instance().get(&TREASURY_KEY)
}

pub fn set_treasury(env: &Env, treasury: &Address) {
    env.storage().instance().set(&TREASURY_KEY, treasury);
}

// --- Merchant Incentive Fund ---

pub fn get_merchant_fund(env: &Env) -> Option<Address> {
    env.storage().instance().get(&MERCHANT_FUND_KEY)
}

pub fn set_merchant_fund(env: &Env, merchant_fund: &Address) {
    env.storage().instance().set(&MERCHANT_FUND_KEY, merchant_fund);
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
    env.storage().instance().get(&TOTAL_LIQUIDITY_KEY).unwrap_or(0)
}

pub fn set_total_liquidity(env: &Env, total: i128) {
    env.storage().instance().set(&TOTAL_LIQUIDITY_KEY, &total);
}

// --- Locked Liquidity ---

pub fn get_locked_liquidity(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&LOCKED_LIQUIDITY_KEY)
        .unwrap_or(0)
}

pub fn set_locked_liquidity(env: &Env, locked: i128) {
    env.storage().instance().set(&LOCKED_LIQUIDITY_KEY, &locked);
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
