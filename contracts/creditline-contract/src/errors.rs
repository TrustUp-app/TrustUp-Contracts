use soroban_sdk::contracterror;

// Error types for the creditline contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CreditLineError {
    NotAdmin = 1,
    InsufficientGuarantee = 2,
    MerchantNotActive = 3,
    InsufficientReputation = 4,
    InsufficientLiquidity = 5,
    LoanNotFound = 6,
    LoanNotActive = 7,
    NotBorrower = 8,
    InvalidAmount = 9,
    Overflow = 10,
    Underflow = 11,
}
