use soroban_sdk::contracterror;

// Error types for the adapter-trustless-contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
    NotAdmin = 1,
    NotSigner = 2,
    AlreadyInitialized = 3,
    EmptySigners = 4,
    InvalidThreshold = 5,
    ActionNotFound = 6,
    ActionAlreadyExecuted = 7,
    ActionCanceled = 8,
    AlreadyApproved = 9,
    ApprovalNotFound = 10,
    ThresholdNotMet = 11,
    TimelockNotElapsed = 12,
    SignerAlreadyExists = 13,
    SignerNotFound = 14,
}
