#![no_std]

mod access;
mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod tests;

use errors::Error;
use soroban_sdk::{contract, contractimpl, Address, Env, String};
use types::MerchantInfo;

#[contract]
pub struct MerchantRegistryContract;

#[contractimpl]
impl MerchantRegistryContract {
    /// Initializes the contract with an admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Registers a new merchant
    pub fn register_merchant(
        env: Env,
        admin: Address,
        merchant: Address,
        name: String,
    ) -> Result<(), Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }

        // ⬇️ Updated to handle the Result
        access::require_admin(&env, &admin)?;

        if storage::has_merchant(&env, &merchant) {
            return Err(Error::MerchantAlreadyRegistered);
        }

        if name.len() == 0 || name.len() > 64 {
            return Err(Error::InvalidName);
        }

        let info = MerchantInfo {
            name: name.clone(),
            registration_date: env.ledger().timestamp(),
            active: true,
            total_sales: 0,
        };

        storage::set_merchant(&env, &merchant, &info);
        storage::increment_merchant_count(&env);
        events::publish_merchant_registered(&env, merchant, name);

        Ok(())
    }

    /// Deactivates an existing merchant
    pub fn deactivate_merchant(env: Env, admin: Address, merchant: Address) -> Result<(), Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }

        access::require_admin(&env, &admin)?;

        let mut info = storage::get_merchant(&env, &merchant).ok_or(Error::MerchantNotFound)?;

        info.active = false;
        storage::set_merchant(&env, &merchant, &info);
        events::publish_merchant_status(&env, merchant, false);

        Ok(())
    }

    /// Activates an existing merchant
    pub fn activate_merchant(env: Env, admin: Address, merchant: Address) -> Result<(), Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }

        access::require_admin(&env, &admin)?;

        let mut info = storage::get_merchant(&env, &merchant).ok_or(Error::MerchantNotFound)?;

        info.active = true;
        storage::set_merchant(&env, &merchant, &info);
        events::publish_merchant_status(&env, merchant, true);

        Ok(())
    }

    pub fn is_active(env: Env, merchant: Address) -> bool {
        if let Some(info) = storage::get_merchant(&env, &merchant) {
            info.active
        } else {
            false
        }
    }

    pub fn get_merchant_info(env: Env, merchant: Address) -> Result<MerchantInfo, Error> {
        storage::get_merchant(&env, &merchant).ok_or(Error::MerchantNotFound)
    }

    pub fn get_merchant_count(env: Env) -> u64 {
        storage::get_merchant_count(&env)
    }
}
