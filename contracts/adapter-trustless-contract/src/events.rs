use soroban_sdk::{symbol_short, Address, Env, Symbol};

// Event topics
const ACTION_PROPOSED: Symbol = symbol_short!("ACTNPROP");
const ACTION_APPROVED: Symbol = symbol_short!("ACTNAPPRV");
const APPROVAL_REVOKED: Symbol = symbol_short!("ACTNREVK");
const ACTION_EXECUTED: Symbol = symbol_short!("ACTNEXEC");
const ACTION_CANCELED: Symbol = symbol_short!("ACTNCANC");
const SIGNER_ADDED: Symbol = symbol_short!("SIGNERADD");
const SIGNER_REMOVED: Symbol = symbol_short!("SIGNERRM");
const THRESHOLD_CHANGED: Symbol = symbol_short!("THRESHCHG");
const TIMELOCK_CHANGED: Symbol = symbol_short!("TMLOCKCHG");
const ADMIN_CHANGED: Symbol = symbol_short!("ADMINCHGD");

/// Emit an action proposed event
pub fn emit_action_proposed(
    env: &Env,
    action_id: u64,
    proposer: &Address,
    target: &Address,
    function: &Symbol,
) {
    env.events()
        .publish((ACTION_PROPOSED, action_id), (proposer, target, function));
}

/// Emit an action approved event
pub fn emit_action_approved(env: &Env, action_id: u64, signer: &Address) {
    env.events().publish((ACTION_APPROVED, action_id), signer);
}

/// Emit an approval revoked event
pub fn emit_approval_revoked(env: &Env, action_id: u64, signer: &Address) {
    env.events().publish((APPROVAL_REVOKED, action_id), signer);
}

/// Emit an action executed event
pub fn emit_action_executed(env: &Env, action_id: u64, target: &Address, function: &Symbol) {
    env.events()
        .publish((ACTION_EXECUTED, action_id), (target, function));
}

/// Emit an action canceled event
pub fn emit_action_canceled(env: &Env, action_id: u64) {
    env.events().publish((ACTION_CANCELED,), action_id);
}

/// Emit a signer added event
pub fn emit_signer_added(env: &Env, signer: &Address) {
    env.events().publish((SIGNER_ADDED,), signer);
}

/// Emit a signer removed event
pub fn emit_signer_removed(env: &Env, signer: &Address) {
    env.events().publish((SIGNER_REMOVED,), signer);
}

/// Emit a threshold changed event
pub fn emit_threshold_changed(env: &Env, old_threshold: u32, new_threshold: u32) {
    env.events()
        .publish((THRESHOLD_CHANGED,), (old_threshold, new_threshold));
}

/// Emit a timelock changed event
pub fn emit_timelock_changed(env: &Env, old_timelock: u64, new_timelock: u64) {
    env.events()
        .publish((TIMELOCK_CHANGED,), (old_timelock, new_timelock));
}

/// Emit an admin changed event
pub fn emit_admin_changed(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events()
        .publish((ADMIN_CHANGED,), (old_admin, new_admin));
}
