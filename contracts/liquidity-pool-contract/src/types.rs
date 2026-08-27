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

/// Parameters of the utilization-based, Aave-style *kinked* interest rate curve.
///
/// All values are basis points (10000 = 100%). `optimal_utilization_bps` is the
/// kink: below it the curve rises with `slope1_bps`, above it with the steeper
/// `slope2_bps`. Utilization `u` is `locked_liquidity / total_liquidity`.
///
/// ```text
/// u <= optimal:  rate = base + slope1 * u
/// u >  optimal:  rate = base + slope1 * optimal + slope2 * (u - optimal)
/// ```
/// (with `u`, `optimal` taken as fractions, i.e. bps / 10000)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateParams {
    pub base_rate_bps: i128,
    pub slope1_bps: i128,
    pub slope2_bps: i128,
    /// Kink point in bps, strictly between 0 and 10000.
    pub optimal_utilization_bps: i128,
}

// Default rate curve applied at initialization: 2% base, +4% up to the kink,
// then a steep +60% slope above 80% utilization.
pub const DEFAULT_BASE_RATE_BPS: i128 = 200;
pub const DEFAULT_SLOPE1_BPS: i128 = 400;
pub const DEFAULT_SLOPE2_BPS: i128 = 6000;
pub const DEFAULT_OPTIMAL_UTILIZATION_BPS: i128 = 8000;
