#![no_std]
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, Env, Symbol, Val, Vec,
};

// Module imports
mod access;
mod errors;
mod events;
mod storage;
mod types;

// Re-export types for external use
pub use errors::AdapterError;
pub use types::Action;

/// Adapter trustless contract structure.
///
/// A generic M-of-N multi-sig + timelock gateway: privileged cross-contract calls
/// are proposed, approved by a threshold of signers, and executed only after a
/// timelock delay has elapsed, so no single signer (including the admin) can act
/// unilaterally.
#[contract]
pub struct AdapterTrustlessContract;

#[contractimpl]
impl AdapterTrustlessContract {
    /// Get the version of this contract
    pub fn get_version() -> Symbol {
        symbol_short!("v1_0_0")
    }

    /// Initialize the adapter with an admin, an initial signer set, an approval
    /// threshold, and a timelock delay (in seconds).
    pub fn initialize(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        timelock_secs: u64,
    ) {
        if storage::has_admin(&env) {
            panic_with_error!(&env, AdapterError::AlreadyInitialized);
        }
        if signers.is_empty() {
            panic_with_error!(&env, AdapterError::EmptySigners);
        }
        if threshold == 0 || threshold > signers.len() {
            panic_with_error!(&env, AdapterError::InvalidThreshold);
        }

        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_signers(&env, &signers);
        storage::set_threshold(&env, threshold);
        storage::set_timelock(&env, timelock_secs);

        events::emit_admin_changed(&env, &admin, &admin);
    }

    /// Propose a cross-contract call. The proposer's approval counts as the first
    /// vote. Returns the newly assigned action id.
    pub fn propose_action(
        env: Env,
        proposer: Address,
        target: Address,
        function: Symbol,
        args: Vec<Val>,
    ) -> u64 {
        proposer.require_auth();
        access::require_signer(&env, &proposer);

        let action_id = storage::next_action_id(&env);

        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer.clone());

        let action = Action {
            id: action_id,
            target: target.clone(),
            function: function.clone(),
            args,
            proposer: proposer.clone(),
            proposed_at: env.ledger().timestamp(),
            approvals,
            executed: false,
            canceled: false,
        };

        storage::set_action(&env, &action);
        events::emit_action_proposed(&env, action_id, &proposer, &target, &function);

        action_id
    }

    /// Approve a proposed action. Requires authorization from an authorized signer.
    pub fn approve_action(env: Env, signer: Address, action_id: u64) {
        signer.require_auth();
        access::require_signer(&env, &signer);

        let mut action = storage::get_action(&env, action_id);
        Self::require_pending(&env, &action);

        if action.approvals.contains(&signer) {
            panic_with_error!(&env, AdapterError::AlreadyApproved);
        }

        action.approvals.push_back(signer.clone());
        storage::set_action(&env, &action);

        events::emit_action_approved(&env, action_id, &signer);
    }

    /// Revoke a previously given approval. Requires authorization from the signer
    /// who approved.
    pub fn revoke_approval(env: Env, signer: Address, action_id: u64) {
        signer.require_auth();

        let mut action = storage::get_action(&env, action_id);
        Self::require_pending(&env, &action);

        match action.approvals.iter().position(|a| a == signer) {
            Some(index) => {
                action.approvals.remove(index as u32);
            }
            None => panic_with_error!(&env, AdapterError::ApprovalNotFound),
        }

        storage::set_action(&env, &action);
        events::emit_approval_revoked(&env, action_id, &signer);
    }

    /// Execute an action once it has reached the approval threshold and the
    /// timelock delay has elapsed. Any address may trigger execution; the
    /// trustless guarantee comes from the approval + timelock checks, not from
    /// who calls this function.
    pub fn execute_action(env: Env, caller: Address, action_id: u64) -> Val {
        caller.require_auth();

        let mut action = storage::get_action(&env, action_id);
        Self::require_pending(&env, &action);

        let threshold = storage::get_threshold(&env);
        if action.approvals.len() < threshold {
            panic_with_error!(&env, AdapterError::ThresholdNotMet);
        }

        let timelock = storage::get_timelock(&env);
        let executable_at = action.proposed_at.saturating_add(timelock);
        if env.ledger().timestamp() < executable_at {
            panic_with_error!(&env, AdapterError::TimelockNotElapsed);
        }

        action.executed = true;
        storage::set_action(&env, &action);

        let result: Val =
            env.invoke_contract(&action.target, &action.function, action.args.clone());

        events::emit_action_executed(&env, action_id, &action.target, &action.function);

        result
    }

    /// Cancel a pending action. Requires authorization from the admin.
    pub fn cancel_action(env: Env, admin: Address, action_id: u64) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        let mut action = storage::get_action(&env, action_id);
        if action.executed {
            panic_with_error!(&env, AdapterError::ActionAlreadyExecuted);
        }

        action.canceled = true;
        storage::set_action(&env, &action);

        events::emit_action_canceled(&env, action_id);
    }

    /// Add an authorized signer. Requires authorization from the admin.
    pub fn add_signer(env: Env, admin: Address, signer: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        let mut signers = storage::get_signers(&env);
        if signers.contains(&signer) {
            panic_with_error!(&env, AdapterError::SignerAlreadyExists);
        }

        signers.push_back(signer.clone());
        storage::set_signers(&env, &signers);

        events::emit_signer_added(&env, &signer);
    }

    /// Remove an authorized signer. Requires authorization from the admin. Fails
    /// if removing the signer would drop the signer count below the current
    /// threshold.
    pub fn remove_signer(env: Env, admin: Address, signer: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        let mut signers = storage::get_signers(&env);
        let index = match signers.iter().position(|s| s == signer) {
            Some(index) => index,
            None => panic_with_error!(&env, AdapterError::SignerNotFound),
        };

        let threshold = storage::get_threshold(&env);
        if signers.len() - 1 < threshold {
            panic_with_error!(&env, AdapterError::InvalidThreshold);
        }

        signers.remove(index as u32);
        storage::set_signers(&env, &signers);

        events::emit_signer_removed(&env, &signer);
    }

    /// Change the approval threshold. Requires authorization from the admin.
    pub fn set_threshold(env: Env, admin: Address, threshold: u32) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        let signers = storage::get_signers(&env);
        if threshold == 0 || threshold > signers.len() {
            panic_with_error!(&env, AdapterError::InvalidThreshold);
        }

        let old_threshold = storage::get_threshold(&env);
        storage::set_threshold(&env, threshold);

        events::emit_threshold_changed(&env, old_threshold, threshold);
    }

    /// Change the timelock delay (in seconds). Requires authorization from the admin.
    pub fn set_timelock(env: Env, admin: Address, timelock_secs: u64) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        let old_timelock = storage::get_timelock(&env);
        storage::set_timelock(&env, timelock_secs);

        events::emit_timelock_changed(&env, old_timelock, timelock_secs);
    }

    /// Set the admin address for this contract. Requires authorization from the
    /// current admin.
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        storage::set_admin(&env, &new_admin);
        events::emit_admin_changed(&env, &admin, &new_admin);
    }

    /// Get a proposed action by id.
    pub fn get_action(env: Env, action_id: u64) -> Action {
        storage::get_action(&env, action_id)
    }

    /// Whether an action has reached the required approval threshold.
    pub fn is_approved(env: Env, action_id: u64) -> bool {
        let action = storage::get_action(&env, action_id);
        action.approvals.len() >= storage::get_threshold(&env)
    }

    /// Whether an action can be executed right now (approved, timelock elapsed,
    /// not already executed or canceled).
    pub fn is_executable(env: Env, action_id: u64) -> bool {
        let action = storage::get_action(&env, action_id);
        if action.executed || action.canceled {
            return false;
        }

        let threshold = storage::get_threshold(&env);
        let timelock = storage::get_timelock(&env);
        let executable_at = action.proposed_at.saturating_add(timelock);

        action.approvals.len() >= threshold && env.ledger().timestamp() >= executable_at
    }

    /// Get the current list of authorized signers.
    pub fn get_signers(env: Env) -> Vec<Address> {
        storage::get_signers(&env)
    }

    /// Get the current approval threshold.
    pub fn get_threshold(env: Env) -> u32 {
        storage::get_threshold(&env)
    }

    /// Get the current timelock delay, in seconds.
    pub fn get_timelock(env: Env) -> u64 {
        storage::get_timelock(&env)
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    fn require_pending(env: &Env, action: &Action) {
        if action.executed {
            panic_with_error!(env, AdapterError::ActionAlreadyExecuted);
        }
        if action.canceled {
            panic_with_error!(env, AdapterError::ActionCanceled);
        }
    }
}

#[cfg(test)]
mod tests;
