use soroban_sdk::{panic_with_error, Address, Env};

use crate::errors::AdapterError;
use crate::storage;

/// Panics unless `caller` is the admin.
pub fn require_admin(env: &Env, caller: &Address) {
    if &storage::get_admin(env) != caller {
        panic_with_error!(env, AdapterError::NotAdmin);
    }
}

/// Panics unless `caller` is the registered CreditLine contract.
pub fn require_creditline(env: &Env, caller: &Address) {
    if &storage::get_creditline(env) != caller {
        panic_with_error!(env, AdapterError::NotCreditLine);
    }
}
