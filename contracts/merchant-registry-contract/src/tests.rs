#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

/// Helper to build a valid MerchantMetadata for tests
fn make_metadata(env: &Env, name: &str, btype: &str, contact: &str) -> MerchantMetadata {
    MerchantMetadata {
        name: String::from_str(env, name),
        business_type: String::from_str(env, btype),
        contact_info: String::from_str(env, contact),
    }
}

/// Helper function to set up the environment, contract, and test addresses.
fn setup<'a>(env: &'a Env) -> (MerchantRegistryContractClient<'a>, Address, Address) {
    let contract_id = env.register(MerchantRegistryContract, ());
    let client = MerchantRegistryContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let merchant = Address::generate(env);

    client.initialize(&admin);

    (client, admin, merchant)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register(MerchantRegistryContract, ());
    let client = MerchantRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // Initial setup succeeds
    client.initialize(&admin);

    // Initializing twice throws an error
    let res = client.try_initialize(&admin);
    assert!(res.is_err());
}

#[test]
fn test_registration_flow() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);

    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let metadata = make_metadata(&env, "Galaxy Tech Supplies", "retail", "galaxy@tech.com");
    client.register_merchant(&admin, &merchant, &metadata);

    assert!(client.is_active(&merchant));

    let info = client.get_merchant_info(&merchant);
    assert_eq!(info.metadata.name, metadata.name);
    assert_eq!(info.metadata.business_type, metadata.business_type);
    assert_eq!(info.metadata.contact_info, metadata.contact_info);
    assert_eq!(info.registration_date, 1_000_000);
    assert_eq!(info.active, true);
    assert_eq!(info.total_sales, 0);
    assert_eq!(client.get_merchant_count(), 1);
}

#[test]
fn test_metadata_stored_correctly() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let metadata = make_metadata(
        &env,
        "Stellar Books",
        "bookstore",
        "https://stellar-books.example",
    );
    client.register_merchant(&admin, &merchant, &metadata);

    let info = client.get_merchant_info(&merchant);
    assert_eq!(info.metadata.name, String::from_str(&env, "Stellar Books"));
    assert_eq!(
        info.metadata.business_type,
        String::from_str(&env, "bookstore")
    );
    assert_eq!(
        info.metadata.contact_info,
        String::from_str(&env, "https://stellar-books.example")
    );
}

#[test]
fn test_duplicate_prevention() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let metadata = make_metadata(&env, "Stellar Books", "retail", "info@stellar.com");
    client.register_merchant(&admin, &merchant, &metadata);

    // Registering the exact same address again should fail
    let res = client.try_register_merchant(&admin, &merchant, &metadata);
    assert!(res.is_err());
}

#[test]
fn test_admin_only_access() {
    let env = Env::default();
    let (client, _admin, merchant) = setup(&env);
    env.mock_all_auths();

    let fake_admin = Address::generate(&env);
    let metadata = make_metadata(&env, "Rogue Merchant", "unknown", "rogue@rogue.com");

    let res = client.try_register_merchant(&fake_admin, &merchant, &metadata);
    assert!(res.is_err());
}

#[test]
fn test_invalid_name_empty() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    // Empty name should be rejected
    let metadata = make_metadata(&env, "", "retail", "contact@shop.com");
    let res = client.try_register_merchant(&admin, &merchant, &metadata);
    assert!(res.is_err());
}

#[test]
fn test_invalid_name_too_long() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    // 65-char name should be rejected
    let long_name = "A".repeat(65);
    let metadata = make_metadata(&env, &long_name, "retail", "contact@shop.com");
    let res = client.try_register_merchant(&admin, &merchant, &metadata);
    assert!(res.is_err());
}

#[test]
fn test_invalid_business_type_too_long() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    // business_type > 64 chars should be rejected
    let long_type = "B".repeat(65);
    let metadata = make_metadata(&env, "Valid Name", &long_type, "contact@shop.com");
    let res = client.try_register_merchant(&admin, &merchant, &metadata);
    assert!(res.is_err());
}

#[test]
fn test_invalid_contact_info_too_long() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    // contact_info > 128 chars should be rejected
    let long_contact = "C".repeat(129);
    let metadata = make_metadata(&env, "Valid Name", "retail", &long_contact);
    let res = client.try_register_merchant(&admin, &merchant, &metadata);
    assert!(res.is_err());
}

#[test]
fn test_activation_and_deactivation() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let metadata = make_metadata(&env, "Nebula Cafe", "food", "nebula@cafe.com");
    client.register_merchant(&admin, &merchant, &metadata);

    assert!(client.is_active(&merchant));

    // Deactivate merchant
    client.deactivate_merchant(&admin, &merchant);
    assert!(!client.is_active(&merchant));
    assert_eq!(client.get_merchant_info(&merchant).active, false);

    // Activate merchant
    client.activate_merchant(&admin, &merchant);
    assert!(client.is_active(&merchant));
    assert_eq!(client.get_merchant_info(&merchant).active, true);
}

#[test]
fn test_set_merchant_status() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let metadata = make_metadata(&env, "Quasar Goods", "wholesale", "quasar@goods.com");
    client.register_merchant(&admin, &merchant, &metadata);

    assert!(client.is_active(&merchant));

    // Deactivate via set_merchant_status
    client.set_merchant_status(&admin, &merchant, &false);
    assert!(!client.is_active(&merchant));

    // Reactivate via set_merchant_status
    client.set_merchant_status(&admin, &merchant, &true);
    assert!(client.is_active(&merchant));

    // Non-admin must be rejected
    let fake_admin = Address::generate(&env);
    let res = client.try_set_merchant_status(&fake_admin, &merchant, &false);
    assert!(res.is_err());
}

#[test]
fn test_merchant_count_increments() {
    let env = Env::default();
    let (client, admin, _) = setup(&env);
    env.mock_all_auths();

    assert_eq!(client.get_merchant_count(), 0);

    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    let m3 = Address::generate(&env);

    client.register_merchant(&admin, &m1, &make_metadata(&env, "Merchant One", "retail", "m1@test.com"));
    assert_eq!(client.get_merchant_count(), 1);

    client.register_merchant(&admin, &m2, &make_metadata(&env, "Merchant Two", "food", "m2@test.com"));
    assert_eq!(client.get_merchant_count(), 2);

    client.register_merchant(&admin, &m3, &make_metadata(&env, "Merchant Three", "services", "m3@test.com"));
    assert_eq!(client.get_merchant_count(), 3);
}

