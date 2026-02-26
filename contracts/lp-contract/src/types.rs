use soroban_sdk::contracttype;

/// Pool statistics returned by get_pool_stats
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolStats {
    pub total_liquidity: i128,
    pub total_shares: i128,
}
