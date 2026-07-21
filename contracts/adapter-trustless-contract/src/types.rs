use soroban_sdk::{contracttype, Address, Symbol, Val, Vec};

/// A proposed cross-contract call, gated by multi-sig approval and a timelock.
#[contracttype]
#[derive(Clone)]
pub struct Action {
    pub id: u64,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub proposer: Address,
    pub proposed_at: u64,
    pub approvals: Vec<Address>,
    pub executed: bool,
    pub canceled: bool,
}

/// Storage keys for the adapter-trustless-contract
#[contracttype]
pub enum DataKey {
    /// Admin address (instance storage)
    Admin,
    /// Authorized signers (instance storage)
    Signers,
    /// Number of approvals required to execute an action (instance storage)
    Threshold,
    /// Minimum delay, in seconds, between proposal and execution (instance storage)
    TimelockSecs,
    /// Next action id to assign (instance storage)
    NextActionId,
    /// Proposed action (persistent storage), keyed by action id
    Action(u64),
}
