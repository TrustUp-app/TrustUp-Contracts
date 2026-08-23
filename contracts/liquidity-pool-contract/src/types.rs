use soroban_sdk::{contracttype, Address};

/// Pool statistics returned by get_pool_stats
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolStats {
    pub total_liquidity: i128,
    pub locked_liquidity: i128,
    pub available_liquidity: i128,
    pub total_shares: i128,
    /// Share price expressed in basis points (10000 = $1.00)
    pub share_price: i128,
}

/// Identifies an allowance granted by `from` to `spender` over LP shares.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceKey {
    pub from: Address,
    pub spender: Address,
}

/// Allowance amount together with the ledger at which it stops being usable.
///
/// Mirrors SEP-41: an allowance is only spendable while
/// `expiration_ledger >= current ledger sequence`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

// Fee split constants (basis points, sum = 10000)
pub const LP_FEE_BPS: i128 = 8500; // 85% to liquidity providers
pub const PROTOCOL_FEE_BPS: i128 = 1000; // 10% to protocol treasury
#[allow(dead_code)]
pub const MERCHANT_FEE_BPS: i128 = 500; // 5% to merchant incentive fund (used as remainder to avoid rounding loss)
pub const TOTAL_BPS: i128 = 10000;

/// Minimum deposit / withdrawal to prevent rounding exploits
pub const MIN_AMOUNT: i128 = 1;
