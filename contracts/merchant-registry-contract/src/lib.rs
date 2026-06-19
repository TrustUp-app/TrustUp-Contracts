#![no_std]

mod access;
mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod tests;

use errors::Error;
use soroban_sdk::{contract, contractimpl, Address, Env};
use types::{MerchantInfo, MerchantMetadata as MerchantMeta};

// Export types for external use
pub use errors::Error as MerchantRegistryError;
pub use types::MerchantMetadata;

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

    /// Registers a new merchant with full metadata.
    ///
    /// # Arguments
    /// * `admin`    — Must match the stored admin address (auth required)
    /// * `merchant` — The on-chain address to be registered
    /// * `metadata` — `MerchantMetadata { name, business_type, contact_info }`
    ///
    /// # Errors
    /// * `NotInitialized`          — Contract has not been initialized
    /// * `Unauthorized`            — Caller is not the admin
    /// * `MerchantAlreadyRegistered` — Address already registered
    /// * `InvalidName`             — name is empty or longer than 64 chars
    /// * `InvalidMetadata`         — business_type or contact_info too long
    pub fn register_merchant(
        env: Env,
        admin: Address,
        merchant: Address,
        metadata: MerchantMeta,
    ) -> Result<(), Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }

        access::require_admin(&env, &admin)?;

        if storage::has_merchant(&env, &merchant) {
            return Err(Error::MerchantAlreadyRegistered);
        }

        // Validate name: 1–64 chars
        if metadata.name.len() == 0 || metadata.name.len() > 64 {
            return Err(Error::InvalidName);
        }

        // Validate business_type: max 64 chars
        if metadata.business_type.len() > 64 {
            return Err(Error::InvalidMetadata);
        }

        // Validate contact_info: max 128 chars
        if metadata.contact_info.len() > 128 {
            return Err(Error::InvalidMetadata);
        }

        let info = MerchantInfo {
            metadata: metadata.clone(),
            registration_date: env.ledger().timestamp(),
            active: true,
            total_sales: 0,
        };

        storage::set_merchant(&env, &merchant, &info);
        storage::increment_merchant_count(&env);
        events::publish_merchant_registered(&env, merchant, metadata);

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

    /// Sets a merchant's active status (admin only).
    /// Pass `active = true` to activate, `active = false` to deactivate.
    pub fn set_merchant_status(
        env: Env,
        admin: Address,
        merchant: Address,
        active: bool,
    ) -> Result<(), Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }

        access::require_admin(&env, &admin)?;

        let mut info = storage::get_merchant(&env, &merchant).ok_or(Error::MerchantNotFound)?;

        info.active = active;
        storage::set_merchant(&env, &merchant, &info);
        events::publish_merchant_status(&env, merchant, active);

        Ok(())
    }

    /// Returns whether the merchant is currently active
    pub fn is_active(env: Env, merchant: Address) -> bool {
        if let Some(info) = storage::get_merchant(&env, &merchant) {
            info.active
        } else {
            false
        }
    }

    /// Returns the full MerchantInfo record for a registered merchant
    pub fn get_merchant_info(env: Env, merchant: Address) -> Result<MerchantInfo, Error> {
        storage::get_merchant(&env, &merchant).ok_or(Error::MerchantNotFound)
    }

    /// Returns the total number of registered merchants
    pub fn get_merchant_count(env: Env) -> u64 {
        storage::get_merchant_count(&env)
    }
}

