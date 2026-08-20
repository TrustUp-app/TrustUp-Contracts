use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

use crate::types::ProtocolParameters;

const PARAMS_UPDATED: Symbol = symbol_short!("PARMUPDT");
const ADMIN_UPDATED: Symbol = symbol_short!("PARMADMN");
const PROPOSED: Symbol = symbol_short!("PRMPROP");
const APPROVED: Symbol = symbol_short!("PRMAPPR");
const EXECUTED: Symbol = symbol_short!("PRMEXEC");
const CANCELLED: Symbol = symbol_short!("PRMCANC");
const PAUSED: Symbol = symbol_short!("PRMPAUS");
const UNPAUSED: Symbol = symbol_short!("PRMUNPAU");
const MIGRATED: Symbol = symbol_short!("PRMMIGR");

pub fn emit_parameters_updated(env: &Env, admin: &Address, params: &ProtocolParameters) {
    env.events().publish(
        (PARAMS_UPDATED, admin),
        (
            params.min_guarantee_percent,
            params.min_reputation_threshold,
            params.full_repayment_reward,
            params.default_penalty,
            params.large_loan_threshold,
            params.large_loan_default_penalty,
            params.base_interest_bps,
        ),
    );
}

pub fn emit_admin_updated(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events()
        .publish((ADMIN_UPDATED, old_admin), new_admin.clone());
}

/// Migrating from single-admin to multi-sig governance.
pub fn emit_migrated(env: &Env, signers: &Vec<Address>, threshold: u32, timelock_secs: u64) {
    env.events()
        .publish((MIGRATED,), (signers.clone(), threshold, timelock_secs));
}

pub fn emit_proposed(env: &Env, proposer: &Address, proposal_id: u64) {
    env.events().publish((PROPOSED, proposer), proposal_id);
}

pub fn emit_approved(env: &Env, signer: &Address, proposal_id: u64) {
    env.events().publish((APPROVED, signer), proposal_id);
}

pub fn emit_executed(env: &Env, caller: &Address, proposal_id: u64) {
    env.events().publish((EXECUTED, caller), proposal_id);
}

pub fn emit_cancelled(env: &Env, caller: &Address, proposal_id: u64) {
    env.events().publish((CANCELLED, caller), proposal_id);
}

pub fn emit_paused(env: &Env, caller: &Address) {
    env.events().publish((PAUSED,), caller.clone());
}

pub fn emit_unpaused(env: &Env, caller: &Address) {
    env.events().publish((UNPAUSED,), caller.clone());
}
