use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ParametersError {
    AlreadyInitialized = 1,
    NotAdmin = 2,
    InvalidParameters = 3,
    // --- governance (proposal / multi-sig / timelock / pause) errors ---
    NotSigner = 4,
    AlreadyApproved = 5,
    ProposalNotFound = 6,
    ProposalNotExecutable = 7,
    ProposalAlreadyFinalized = 8,
    InvalidThreshold = 9,
    AlreadyPaused = 10,
    NotPaused = 11,
    /// `update_parameters` / `set_admin` are disabled once the signer set is
    /// configured; changes must go through the proposal workflow instead.
    GovernanceActive = 12,
}
