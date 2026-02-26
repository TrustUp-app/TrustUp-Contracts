#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Env};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::LiquidityPoolError;
pub use types::PoolStats;

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolContract {
    /// Initialize the contract. Can only be called once.
    ///
    /// * `admin` – Contract administrator
    /// * `token` – SEP-41 token used by the pool (e.g. USDC)
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if storage::has_admin(&env) {
            panic_with_error!(&env, LiquidityPoolError::AlreadyInitialized);
        }
        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
    }

    /// Deposit `amount` tokens and receive shares representing pool ownership.
    ///
    /// **First deposit**: shares issued == amount (1:1 ratio).
    /// **Subsequent deposits**: `shares = (amount × total_shares) / total_pool_value`
    ///
    /// Returns the number of shares issued.
    pub fn deposit(env: Env, provider: Address, amount: i128) -> i128 {
        provider.require_auth();

        // Reject zero or negative deposits
        if amount <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        let token = storage::get_token(&env);
        let total_shares = storage::get_total_shares(&env);
        let total_liquidity = storage::get_total_liquidity(&env);

        let shares_issued = if total_shares == 0 || total_liquidity == 0 {
            // First deposit: 1:1 ratio
            amount
        } else {
            amount
                .checked_mul(total_shares)
                .and_then(|v| v.checked_div(total_liquidity))
                .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow))
        };

        // Guard: tiny deposits that round down to zero shares are rejected
        if shares_issued <= 0 {
            panic_with_error!(&env, LiquidityPoolError::InvalidAmount);
        }

        // Transfer tokens from provider → pool contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&provider, &env.current_contract_address(), &amount);

        // Mint shares: update provider balance
        let new_provider_shares = storage::get_lp_shares(&env, &provider)
            .checked_add(shares_issued)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_lp_shares(&env, &provider, new_provider_shares);

        // Update global totals
        let new_total_shares = total_shares
            .checked_add(shares_issued)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_shares(&env, new_total_shares);

        let new_total_liquidity = total_liquidity
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, LiquidityPoolError::Overflow));
        storage::set_total_liquidity(&env, new_total_liquidity);

        events::emit_liquidity_deposited(&env, &provider, amount, shares_issued);

        shares_issued
    }

    pub fn get_pool_stats(env: Env) -> PoolStats {
        PoolStats {
            total_liquidity: storage::get_total_liquidity(&env),
            total_shares: storage::get_total_shares(&env),
        }
    }

    pub fn get_lp_shares(env: Env, provider: Address) -> i128 {
        storage::get_lp_shares(&env, &provider)
    }
}

#[cfg(test)]
mod tests;
