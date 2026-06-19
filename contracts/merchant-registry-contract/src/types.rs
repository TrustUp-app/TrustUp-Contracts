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

/// Metadata provided at registration time by the admin.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantMetadata {
    /// Display name of the merchant (1–64 chars)
    pub name: String,
    /// Category or type of business (e.g. "retail", "services")
    pub business_type: String,
    /// Contact information (e.g. email or URL, max 128 chars)
    pub contact_info: String,
}

/// Full on-chain record for a registered merchant.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantInfo {
    pub metadata: MerchantMetadata,
    pub registration_date: u64,
    pub active: bool,
    pub total_sales: u64,
}

