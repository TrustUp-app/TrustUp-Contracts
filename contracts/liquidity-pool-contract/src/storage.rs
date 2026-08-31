use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::{AllowanceKey, AllowanceValue, RateParams};

// Instance storage keys
pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const TOKEN_KEY: Symbol = symbol_short!("TOKEN");
pub const TOTAL_SHARES_KEY: Symbol = symbol_short!("TOTSHRS");
pub const TOTAL_LIQUIDITY_KEY: Symbol = symbol_short!("TOTLIQ");
pub const LOCKED_LIQUIDITY_KEY: Symbol = symbol_short!("LCKDLIQ");
pub const CREDITLINE_KEY: Symbol = symbol_short!("CRDTLIN");
pub const TREASURY_KEY: Symbol = symbol_short!("TREASURY");
pub const MERCHANT_FUND_KEY: Symbol = symbol_short!("MRCHFND");
pub const REENTRANCY_LOCK_KEY: Symbol = symbol_short!("LOCKED");
pub const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
pub const RATE_PARAMS_KEY: Symbol = symbol_short!("RATEPARM");
pub const PARAMETERS_CONTRACT_KEY: Symbol = symbol_short!("PARAMS");

// Persistent storage key prefix for LP shares
pub const LP_SHARES_PREFIX: Symbol = symbol_short!("LPSHRS");

// Temporary storage key prefix for share allowances (owner → spender)
pub const ALLOWANCE_PREFIX: Symbol = symbol_short!("LPALLOW");

// TTL constants (~30 days at 5 s/ledger)
const INSTANCE_BUMP_AMOUNT: u32 = 518_400;
const INSTANCE_LIFETIME_THRESHOLD: u32 = 259_200;
const PERSISTENT_BUMP_AMOUNT: u32 = 518_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 259_200;

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn bump_lp_shares(env: &Env, provider: &Address) {
    let key = (LP_SHARES_PREFIX, provider.clone());
    // extend_ttl panics on a missing entry, so only bump providers that have one.
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
}

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
    env.storage()
        .instance()
        .set(&MERCHANT_FUND_KEY, merchant_fund);
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

// --- Share Allowances (temporary, expire with the allowance itself) ---

fn allowance_key(from: &Address, spender: &Address) -> (Symbol, AllowanceKey) {
    (
        ALLOWANCE_PREFIX,
        AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        },
    )
}

/// Read the live allowance `from` granted `spender`.
///
/// An allowance whose `expiration_ledger` has passed reads as zero, matching
/// SEP-41 semantics, so an expired approval can never be spent.
pub fn get_allowance(env: &Env, from: &Address, spender: &Address) -> AllowanceValue {
    let key = allowance_key(from, spender);
    match env.storage().temporary().get::<_, AllowanceValue>(&key) {
        Some(allowance) if allowance.expiration_ledger >= env.ledger().sequence() => allowance,
        Some(allowance) => AllowanceValue {
            amount: 0,
            expiration_ledger: allowance.expiration_ledger,
        },
        None => AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        },
    }
}

/// Write an allowance and keep its storage entry alive until it expires.
///
/// Callers must validate `expiration_ledger` before calling this
/// (see `LiquidityPoolContract::approve`).
pub fn set_allowance(
    env: &Env,
    from: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let key = allowance_key(from, spender);
    env.storage().temporary().set(
        &key,
        &AllowanceValue {
            amount,
            expiration_ledger,
        },
    );

    // A zero allowance needs no TTL extension: it reads as zero either way.
    if amount > 0 {
        let live_for = expiration_ledger.saturating_sub(env.ledger().sequence());
        if live_for > 0 {
            env.storage()
                .temporary()
                .extend_ttl(&key, live_for, live_for);
        }
    }
}

// --- Interest Rate Curve Parameters ---

pub fn get_rate_params(env: &Env) -> Option<RateParams> {
    env.storage().instance().get(&RATE_PARAMS_KEY)
}

pub fn set_rate_params(env: &Env, params: &RateParams) {
    env.storage().instance().set(&RATE_PARAMS_KEY, params);
}

pub fn is_reentrancy_locked(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&REENTRANCY_LOCK_KEY)
        .unwrap_or(false)
}

pub fn set_reentrancy_locked(env: &Env, locked: bool) {
    env.storage().instance().set(&REENTRANCY_LOCK_KEY, &locked);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED_KEY, &paused);
}

// --- Parameters Contract ---

pub fn get_parameters_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&PARAMETERS_CONTRACT_KEY)
}

pub fn set_parameters_contract(env: &Env, address: &Address) {
    env.storage().instance().set(&PARAMETERS_CONTRACT_KEY, address);
}
