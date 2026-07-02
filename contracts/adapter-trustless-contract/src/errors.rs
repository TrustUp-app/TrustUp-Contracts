use soroban_sdk::contracterror;

/// Error codes for the adapter-trustless-contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
    /// Caller is not the admin.
    NotAdmin = 1,
    /// Caller is not the registered CreditLine contract.
    NotCreditLine = 2,
    /// No escrow entry exists for the given loan ID.
    EscrowNotFound = 3,
    /// The escrow entry is not in the expected status for this operation.
    InvalidStatus = 4,
    /// Token amount must be greater than zero.
    InvalidAmount = 5,
    /// Contract has already been initialised.
    AlreadyInitialized = 6,
}
