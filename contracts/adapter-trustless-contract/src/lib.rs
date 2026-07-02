#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Env};

mod access;
mod errors;
mod events;
mod storage;
mod types;

pub use errors::AdapterError;
pub use types::{EscrowEntry, EscrowStatus};

/// Trustless escrow adapter contract.
///
/// Holds borrower guarantee deposits for the duration of a loan and
/// releases or seizes them on instruction from the registered CreditLine contract.
#[contract]
pub struct AdapterTrustlessContract;

#[contractimpl]
impl AdapterTrustlessContract {
    /// Initialise the contract.
    ///
    /// Must be called exactly once before any other function.
    pub fn initialize(env: Env, admin: Address, creditline: Address, token: Address) {
        if storage::has_admin(&env) {
            panic_with_error!(&env, AdapterError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_creditline(&env, &creditline);
        storage::set_token(&env, &token);
    }

    // ── Admin operations ────────────────────────────────────────────────────

    /// Transfer admin to a new address.
    pub fn set_admin(env: Env, new_admin: Address) {
        let current = storage::get_admin(&env);
        current.require_auth();
        access::require_admin(&env, &current);
        storage::set_admin(&env, &new_admin);
    }

    /// Update the registered CreditLine contract address.
    ///
    /// Requires admin authorisation.
    pub fn set_creditline(env: Env, admin: Address, creditline: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        storage::set_creditline(&env, &creditline);
    }

    // ── Escrow lifecycle (called by CreditLine) ──────────────────────────────

    /// Lock `amount` tokens from `borrower` as guarantee for `loan_id`.
    ///
    /// Transfers tokens from `borrower` to this contract and records the escrow.
    /// Requires authorisation from the registered CreditLine contract.
    pub fn lock_guarantee(
        env: Env,
        creditline: Address,
        loan_id: u64,
        borrower: Address,
        amount: i128,
    ) {
        creditline.require_auth();
        access::require_creditline(&env, &creditline);

        if amount <= 0 {
            panic_with_error!(&env, AdapterError::InvalidAmount);
        }
        if storage::get_escrow(&env, loan_id).is_some() {
            panic_with_error!(&env, AdapterError::InvalidStatus);
        }

        let token_client = token::Client::new(&env, &storage::get_token(&env));
        token_client.transfer(&borrower, &env.current_contract_address(), &amount);

        storage::set_escrow(
            &env,
            loan_id,
            &EscrowEntry {
                borrower: borrower.clone(),
                amount,
                status: EscrowStatus::Locked,
            },
        );

        events::emit_locked(&env, loan_id, &borrower, amount);
    }

    /// Release the guarantee back to the borrower after successful repayment.
    ///
    /// Requires authorisation from the registered CreditLine contract.
    pub fn release_guarantee(env: Env, creditline: Address, loan_id: u64) {
        creditline.require_auth();
        access::require_creditline(&env, &creditline);

        let mut entry = storage::get_escrow(&env, loan_id)
            .unwrap_or_else(|| panic_with_error!(&env, AdapterError::EscrowNotFound));

        if entry.status != EscrowStatus::Locked {
            panic_with_error!(&env, AdapterError::InvalidStatus);
        }

        let token_client = token::Client::new(&env, &storage::get_token(&env));
        token_client.transfer(&env.current_contract_address(), &entry.borrower, &entry.amount);

        entry.status = EscrowStatus::Released;
        storage::set_escrow(&env, loan_id, &entry);

        events::emit_released(&env, loan_id, &entry.borrower, entry.amount);
    }

    /// Seize the guarantee and forward it to the liquidity pool on default.
    ///
    /// Requires authorisation from the registered CreditLine contract.
    pub fn seize_guarantee(env: Env, creditline: Address, loan_id: u64, pool: Address) {
        creditline.require_auth();
        access::require_creditline(&env, &creditline);

        let mut entry = storage::get_escrow(&env, loan_id)
            .unwrap_or_else(|| panic_with_error!(&env, AdapterError::EscrowNotFound));

        if entry.status != EscrowStatus::Locked {
            panic_with_error!(&env, AdapterError::InvalidStatus);
        }

        let token_client = token::Client::new(&env, &storage::get_token(&env));
        token_client.transfer(&env.current_contract_address(), &pool, &entry.amount);

        entry.status = EscrowStatus::Seized;
        storage::set_escrow(&env, loan_id, &entry);

        events::emit_seized(&env, loan_id, &pool, entry.amount);
    }

    // ── Queries ─────────────────────────────────────────────────────────────

    /// Return the escrow entry for a given loan ID.
    pub fn get_escrow(env: Env, loan_id: u64) -> EscrowEntry {
        storage::get_escrow(&env, loan_id)
            .unwrap_or_else(|| panic_with_error!(&env, AdapterError::EscrowNotFound))
    }

    /// Return the current admin address.
    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    /// Return the registered CreditLine contract address.
    pub fn get_creditline(env: Env) -> Address {
        storage::get_creditline(&env)
    }
}

#[cfg(test)]
mod tests;
