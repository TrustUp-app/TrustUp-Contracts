use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    // Instance storage
    Admin,

    // Persistent storage
    Merchant(Address),
    MerchantCount,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantInfo {
    pub name: String,
    pub registration_date: u64,
    pub active: bool,
    pub total_sales: u64,
}
