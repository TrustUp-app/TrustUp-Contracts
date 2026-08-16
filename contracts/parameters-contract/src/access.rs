use soroban_sdk::{panic_with_error, Address, Env};

use crate::{errors::ParametersError, storage};

pub fn require_admin(env: &Env, caller: &Address) {
    let admin = storage::get_admin(env);
    if admin != *caller {
        panic_with_error!(env, ParametersError::NotAdmin);
    }
}

/// Require that `caller` is a member of the governance signer set.
pub fn require_signer(env: &Env, caller: &Address) {
    if !storage::is_signer(env, caller) {
        panic_with_error!(env, ParametersError::NotSigner);
    }
}
