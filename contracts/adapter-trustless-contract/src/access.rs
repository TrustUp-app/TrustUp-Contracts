use soroban_sdk::{panic_with_error, Address, Env};

use crate::errors::AdapterError;
use crate::storage;

/// Require that the given address is the admin, otherwise panic with NotAdmin error
pub fn require_admin(env: &Env, caller: &Address) {
    let admin = storage::get_admin(env);

    if caller != &admin {
        panic_with_error!(env, AdapterError::NotAdmin);
    }
}

/// Require that the given address is an authorized signer, otherwise panic with NotSigner error
pub fn require_signer(env: &Env, addr: &Address) {
    if !storage::is_signer(env, addr) {
        panic_with_error!(env, AdapterError::NotSigner);
    }
}
