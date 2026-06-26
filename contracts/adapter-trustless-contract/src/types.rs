use soroban_sdk::{contracttype, Address};

/// Status of a guarantee escrow entry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Funds are held in escrow.
    Locked,
    /// Funds were returned to the borrower after successful repayment.
    Released,
    /// Funds were forwarded to the liquidity pool after default.
    Seized,
}

/// A single guarantee escrow record, keyed by `loan_id`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowEntry {
    /// The borrower whose guarantee is held.
    pub borrower: Address,
    /// The locked token amount.
    pub amount: i128,
    /// Current lifecycle status.
    pub status: EscrowStatus,
}
