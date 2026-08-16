#![no_std]

mod access;
mod errors;
mod events;
mod storage;
mod types;

pub use errors::ParametersError;
pub use types::{default_parameters, Proposal, ProposalKind, ProposalStatus, ProtocolParameters};

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Vec};

#[contract]
pub struct ParametersContract;

#[contractimpl]
impl ParametersContract {
    pub fn initialize(env: Env, admin: Address, params: ProtocolParameters) {
        if storage::has_admin(&env) {
            panic_with_error!(&env, ParametersError::AlreadyInitialized);
        }

        Self::validate_parameters(&env, &params);
        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_parameters(&env, &params);
        events::emit_parameters_updated(&env, &admin, &params);
    }

    pub fn initialize_defaults(env: Env, admin: Address) {
        Self::initialize(env, admin, default_parameters());
    }

    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let old_admin = storage::get_admin(&env);
        old_admin.require_auth();
        access::require_admin(&env, &old_admin);
        Self::require_governance_inactive(&env);

        storage::set_admin(&env, &new_admin);
        events::emit_admin_updated(&env, &old_admin, &new_admin);
    }

    pub fn get_parameters(env: Env) -> ProtocolParameters {
        storage::get_parameters(&env)
    }

    /// Legacy single-admin update path. Disabled once `migrate_to_multisig`
    /// has run; changes must then go through `propose_parameters`.
    pub fn update_parameters(env: Env, admin: Address, params: ProtocolParameters) {
        admin.require_auth();
        access::require_admin(&env, &admin);
        Self::require_governance_inactive(&env);
        Self::validate_parameters(&env, &params);

        storage::set_parameters(&env, &params);
        events::emit_parameters_updated(&env, &admin, &params);
    }

    // ─── governance: migration ──────────────────────────────────────────

    /// One-time migration from the single hardcoded `admin` to a governed
    /// M-of-N signer set. Callable only by the current admin, only once.
    pub fn migrate_to_multisig(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        timelock_secs: u64,
    ) {
        admin.require_auth();
        access::require_admin(&env, &admin);

        if storage::has_signers(&env) {
            panic_with_error!(&env, ParametersError::AlreadyInitialized);
        }
        Self::validate_threshold(&env, &signers, threshold);

        storage::set_signers(&env, &signers);
        storage::set_threshold(&env, threshold);
        storage::set_timelock(&env, timelock_secs);
        events::emit_migrated(&env, &signers, threshold, timelock_secs);
    }

    // ─── governance: proposal lifecycle ─────────────────────────────────

    /// Propose a new parameter set. The proposer's approval is recorded
    /// immediately. Returns the new proposal id.
    pub fn propose_parameters(env: Env, proposer: Address, params: ProtocolParameters) -> u64 {
        proposer.require_auth();
        access::require_signer(&env, &proposer);
        Self::validate_parameters(&env, &params);

        Self::create_proposal(&env, proposer, ProposalKind::UpdateParameters(params))
    }

    /// Propose flipping the protocol's paused flag. Pause proposals skip the
    /// timelock on execution so an emergency response isn't delayed.
    pub fn propose_pause(env: Env, proposer: Address, paused: bool) -> u64 {
        proposer.require_auth();
        access::require_signer(&env, &proposer);

        if storage::is_paused(&env) == paused {
            let err = if paused {
                ParametersError::AlreadyPaused
            } else {
                ParametersError::NotPaused
            };
            panic_with_error!(&env, err);
        }

        Self::create_proposal(&env, proposer, ProposalKind::SetPaused(paused))
    }

    fn create_proposal(env: &Env, proposer: Address, kind: ProposalKind) -> u64 {
        let id = storage::next_proposal_id(env);

        let mut approvals = Vec::new(env);
        approvals.push_back(proposer.clone());

        let proposal = Proposal {
            id,
            kind,
            proposer: proposer.clone(),
            proposed_at: env.ledger().timestamp(),
            approvals,
            status: ProposalStatus::Pending,
        };

        storage::set_proposal(env, &proposal);
        events::emit_proposed(env, &proposer, id);
        id
    }

    /// Approve a pending proposal. Each signer may approve once.
    pub fn approve_proposal(env: Env, signer: Address, proposal_id: u64) {
        signer.require_auth();
        access::require_signer(&env, &signer);

        let mut proposal = storage::get_proposal(&env, proposal_id);
        Self::require_pending(&env, &proposal);

        if proposal.approvals.contains(&signer) {
            panic_with_error!(&env, ParametersError::AlreadyApproved);
        }

        proposal.approvals.push_back(signer.clone());
        storage::set_proposal(&env, &proposal);
        events::emit_approved(&env, &signer, proposal_id);
    }

    /// Execute a proposal once it has reached the approval threshold and,
    /// for parameter changes, the timelock has elapsed. Anyone can call this.
    pub fn execute_proposal(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();

        let mut proposal = storage::get_proposal(&env, proposal_id);
        Self::require_pending(&env, &proposal);

        if proposal.approvals.len() < storage::get_threshold(&env) {
            panic_with_error!(&env, ParametersError::ProposalNotExecutable);
        }

        // Parameter changes wait out the timelock; pauses take effect at once.
        if let ProposalKind::UpdateParameters(_) = proposal.kind {
            let ready_at = proposal.proposed_at.saturating_add(storage::get_timelock(&env));
            if env.ledger().timestamp() < ready_at {
                panic_with_error!(&env, ParametersError::ProposalNotExecutable);
            }
        }

        match &proposal.kind {
            ProposalKind::UpdateParameters(params) => {
                storage::set_parameters(&env, params);
                events::emit_parameters_updated(&env, &caller, params);
            }
            ProposalKind::SetPaused(paused) => {
                storage::set_paused(&env, *paused);
                if *paused {
                    events::emit_paused(&env, &caller);
                } else {
                    events::emit_unpaused(&env, &caller);
                }
            }
        }

        proposal.status = ProposalStatus::Executed;
        storage::set_proposal(&env, &proposal);
        events::emit_executed(&env, &caller, proposal_id);
    }

    /// Withdraw a pending proposal. Callable by its proposer or the admin.
    pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();

        let mut proposal = storage::get_proposal(&env, proposal_id);
        Self::require_pending(&env, &proposal);

        if caller != proposal.proposer && caller != storage::get_admin(&env) {
            panic_with_error!(&env, ParametersError::NotSigner);
        }

        proposal.status = ProposalStatus::Cancelled;
        storage::set_proposal(&env, &proposal);
        events::emit_cancelled(&env, &caller, proposal_id);
    }

    // ─── governance: queries ─────────────────────────────────────────────

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        storage::get_proposal(&env, proposal_id)
    }

    pub fn get_signers(env: Env) -> Vec<Address> {
        storage::get_signers(&env)
    }

    pub fn get_threshold(env: Env) -> u32 {
        storage::get_threshold(&env)
    }

    pub fn get_timelock(env: Env) -> u64 {
        storage::get_timelock(&env)
    }

    // ─── internal helpers ────────────────────────────────────────────────

    fn require_pending(env: &Env, proposal: &Proposal) {
        if proposal.status != ProposalStatus::Pending {
            panic_with_error!(env, ParametersError::ProposalAlreadyFinalized);
        }
    }

    /// Blocks the legacy single-admin path once a signer set is configured.
    fn require_governance_inactive(env: &Env) {
        if storage::has_signers(env) {
            panic_with_error!(env, ParametersError::GovernanceActive);
        }
    }

    fn validate_threshold(env: &Env, signers: &Vec<Address>, threshold: u32) {
        if signers.is_empty() || threshold == 0 || threshold > signers.len() {
            panic_with_error!(env, ParametersError::InvalidThreshold);
        }
    }

    fn validate_parameters(env: &Env, params: &ProtocolParameters) {
        if params.min_guarantee_percent <= 0
            || params.min_guarantee_percent > 100
            || params.large_loan_threshold <= 0
        {
            panic_with_error!(env, ParametersError::InvalidParameters);
        }
    }
}

#[cfg(test)]
mod tests;
