use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum LiquidityPoolError {
    AlreadyInitialized = 1, // initialize() called twice
    InvalidAmount = 2,      // amount <= 0 or shares round down to zero
    Overflow = 3,           // arithmetic overflow in share calculation
}
