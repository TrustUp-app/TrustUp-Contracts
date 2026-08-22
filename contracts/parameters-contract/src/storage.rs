use soroban_sdk::{panic_with_error, symbol_short, Address, Env, Symbol, Vec};

use crate::errors::ParametersError;
use crate::types::{DataKey, Proposal, ProtocolParameters};

/// Persistent TTL for proposals: bumped ~30 days when accessed, extended if
/// fewer than ~15 days of TTL remain.
const PROPOSAL_TTL_THRESHOLD: u32 = 259_200;
const PROPOSAL_TTL_EXTEND_TO: u32 = 518_400;

pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const PARAMS_KEY: Symbol = symbol_short!("PARAMS");

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&ADMIN_KEY)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .expect("parameters admin not set")
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN_KEY, admin);
}

pub fn get_parameters(env: &Env) -> ProtocolParameters {
    env.storage()
        .instance()
        .get(&PARAMS_KEY)
        .expect("parameters not set")
}

pub fn set_parameters(env: &Env, params: &ProtocolParameters) {
    env.storage().instance().set(&PARAMS_KEY, params);
}

// --- governance: signer set / threshold / timelock ---

pub fn has_signers(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Signers)
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
    env.storage().instance().get(&DataKey::Threshold).unwrap_or(0)
}

pub fn set_threshold(env: &Env, threshold: u32) {
    env.storage().instance().set(&DataKey::Threshold, &threshold);
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

// --- governance: proposals ---

/// Allocate and persist the next proposal id, starting from 1.
pub fn next_proposal_id(env: &Env) -> u64 {
    let next: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextProposalId)
        .unwrap_or(0)
        + 1;

    env.storage().instance().set(&DataKey::NextProposalId, &next);
    next
}

pub fn get_proposal(env: &Env, proposal_id: u64) -> Proposal {
    let key = DataKey::Proposal(proposal_id);

    let proposal: Proposal = match env.storage().persistent().get(&key) {
        Some(proposal) => proposal,
        None => panic_with_error!(env, ParametersError::ProposalNotFound),
    };

    env.storage()
        .persistent()
        .extend_ttl(&key, PROPOSAL_TTL_THRESHOLD, PROPOSAL_TTL_EXTEND_TO);

    proposal
}

pub fn set_proposal(env: &Env, proposal: &Proposal) {
    let key = DataKey::Proposal(proposal.id);
    env.storage().persistent().set(&key, proposal);
    env.storage()
        .persistent()
        .extend_ttl(&key, PROPOSAL_TTL_THRESHOLD, PROPOSAL_TTL_EXTEND_TO);
}

// --- governance: emergency pause ---

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}
