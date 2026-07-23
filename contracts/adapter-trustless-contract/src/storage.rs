use soroban_sdk::{panic_with_error, Address, Env, Vec};

use crate::errors::AdapterError;
use crate::types::{Action, DataKey};

/// Persistent storage TTL for actions: ~30 days bump when accessed, extended if
/// fewer than ~15 days remain.
const ACTION_TTL_THRESHOLD: u32 = 259_200;
const ACTION_TTL_EXTEND_TO: u32 = 518_400;

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic!("Admin not set"))
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_signers(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Signers)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_signers(env: &Env, signers: &Vec<Address>) {
    env.storage().instance().set(&DataKey::Signers, signers);
}

pub fn is_signer(env: &Env, addr: &Address) -> bool {
    get_signers(env).contains(addr)
}

pub fn get_threshold(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::Threshold)
        .unwrap_or(0)
}

pub fn set_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&DataKey::Threshold, &threshold);
}

pub fn get_timelock(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::TimelockSecs)
        .unwrap_or(0)
}

pub fn set_timelock(env: &Env, timelock_secs: u64) {
    env.storage()
        .instance()
        .set(&DataKey::TimelockSecs, &timelock_secs);
}

/// Allocate and persist the next action id, starting from 1.
pub fn next_action_id(env: &Env) -> u64 {
    let next: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextActionId)
        .unwrap_or(0)
        + 1;

    env.storage().instance().set(&DataKey::NextActionId, &next);
    next
}

pub fn get_action(env: &Env, action_id: u64) -> Action {
    let key = DataKey::Action(action_id);

    let action: Action = match env.storage().persistent().get(&key) {
        Some(action) => action,
        None => panic_with_error!(env, AdapterError::ActionNotFound),
    };

    env.storage()
        .persistent()
        .extend_ttl(&key, ACTION_TTL_THRESHOLD, ACTION_TTL_EXTEND_TO);

    action
}

pub fn set_action(env: &Env, action: &Action) {
    let key = DataKey::Action(action.id);
    env.storage().persistent().set(&key, action);
    env.storage()
        .persistent()
        .extend_ttl(&key, ACTION_TTL_THRESHOLD, ACTION_TTL_EXTEND_TO);
}
