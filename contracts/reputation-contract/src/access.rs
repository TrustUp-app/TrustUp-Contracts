use soroban_sdk::{panic_with_error, Address, Env};

use crate::errors::ReputationError;
use crate::storage;

/// Require that the given address is the admin, otherwise panic with NotAdmin error
pub fn require_admin(env: &Env, caller: &Address) {
    let admin = storage::get_admin(env);

    if caller != &admin {
        panic_with_error!(env, ReputationError::NotAdmin);
    }
}

/// Require that the given address is an authorized updater, otherwise panic with NotUpdater error
pub fn require_updater(env: &Env, addr: &Address) {
    if !storage::is_updater(env, addr) {
        panic_with_error!(env, ReputationError::NotUpdater);
    }
}
