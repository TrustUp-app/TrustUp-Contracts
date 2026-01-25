use soroban_sdk::{contracttype, Address};

// Loan status enum
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoanStatus {
    Active,
    Paid,
    Defaulted,
}

// Repayment installment structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepaymentInstallment {
    pub due_date: u64, // Unix timestamp
    pub amount: i128,  // Amount due for this installment
}

// Loan data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Loan {
    pub loan_id: u64,
    pub borrower: Address,
    pub merchant: Address,
    pub total_amount: i128,
    pub guarantee_amount: i128,
    pub remaining_balance: i128,
    pub repayment_schedule: soroban_sdk::Vec<RepaymentInstallment>,
    pub status: LoanStatus,
    pub created_at: u64, // Unix timestamp
}

// Constants
pub const MIN_GUARANTEE_PERCENT: i128 = 20; // 20% minimum guarantee
pub const MIN_REPUTATION_THRESHOLD: u32 = 50; // Minimum reputation score required
