#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

// Module imports
mod types;
mod storage;
mod access;
mod events;
mod errors;

// Re-export types for external use
pub use errors::ReputationError;

/// Reputation contract structure
#[contract]
pub struct ReputationContract;

/// Contract implementation
#[contractimpl]
impl ReputationContract {
    /// Get the version of this contract
    pub fn get_version() -> Symbol {
        symbol_short!("v1_0_0")
    }

    /// Get the reputation score for a user
    pub fn get_score(env: Env, user: Address) -> u32 {
        storage::read_score(&env, &user)
    }

    /// Increase a user's reputation score by a given amount
    /// Requires authorization from an updater
    pub fn increase_score(env: Env, updater: Address, user: Address, amount: u32) {
        updater.require_auth();
        access::require_updater(&env, &updater);

        let old_score = storage::read_score(&env, &user);
        let new_score = old_score
            .checked_add(amount)
            .ok_or_else(|| ReputationError::Overflow)
            .map_err(|e| {
                soroban_sdk::panic_with_error!(&env, e);
            })
            .unwrap();

        if new_score > types::MAX_SCORE {
            soroban_sdk::panic_with_error!(&env, ReputationError::Overflow);
        }

        storage::write_score(&env, &user, new_score);
        
        let reason = symbol_short!("increase");
        events::emit_score_changed(&env, &user, old_score, new_score, &reason);
    }

    /// Decrease a user's reputation score by a given amount
    /// Requires authorization from an updater
    pub fn decrease_score(env: Env, updater: Address, user: Address, amount: u32) {
        updater.require_auth();
        access::require_updater(&env, &updater);

        let old_score = storage::read_score(&env, &user);
        let new_score = old_score
            .checked_sub(amount)
            .ok_or_else(|| ReputationError::Underflow)
            .map_err(|e| {
                soroban_sdk::panic_with_error!(&env, e);
            })
            .unwrap();

        if new_score < types::MIN_SCORE {
            soroban_sdk::panic_with_error!(&env, ReputationError::Underflow);
        }

        storage::write_score(&env, &user, new_score);
        
        let reason = symbol_short!("decrease");
        events::emit_score_changed(&env, &user, old_score, new_score, &reason);
    }

    /// Set a user's reputation score to a specific value
    /// Requires authorization from an updater
    pub fn set_score(env: Env, updater: Address, user: Address, new_score: u32) {
        updater.require_auth();
        access::require_updater(&env, &updater);

        if new_score < types::MIN_SCORE || new_score > types::MAX_SCORE {
            soroban_sdk::panic_with_error!(&env, ReputationError::OutOfBounds);
        }

        let old_score = storage::read_score(&env, &user);
        storage::write_score(&env, &user, new_score);
        
        let reason = symbol_short!("set");
        events::emit_score_changed(&env, &user, old_score, new_score, &reason);
    }

    /// Set or remove an address as an authorized updater
    /// Requires authorization from admin
    pub fn set_updater(env: Env, admin: Address, updater: Address, allowed: bool) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        
        storage::set_updater(&env, &updater, allowed);
        events::emit_updater_changed(&env, &updater, allowed);
    }

    /// Check if an address is an authorized updater
    pub fn is_updater(env: Env, addr: Address) -> bool {
        storage::is_updater(&env, &addr)
    }

    /// Set the admin address for this contract
    /// Requires authorization from current admin (or allows initial setup)
    pub fn set_admin(env: Env, new_admin: Address) {
        let old_admin_opt: Option<Address> = env.storage().instance().get(&storage::ADMIN_KEY);
        
        if let Some(old_admin) = old_admin_opt {
            // Admin exists, require current admin authorization
            old_admin.require_auth();
            access::require_admin(&env, &old_admin);
            storage::set_admin(&env, &new_admin);
            events::emit_admin_changed(&env, &old_admin, &new_admin);
        } else {
            // No admin exists, allow setting (initialization)
            storage::set_admin(&env, &new_admin);
            let dummy = new_admin.clone();
            events::emit_admin_changed(&env, &dummy, &new_admin);
        }
    }

    /// Get the current admin address
    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
    }
}

#[cfg(test)]
mod tests;
