#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

/// Helper function to set up the environment, contract, and test addresses.
fn setup<'a>(env: &'a Env) -> (MerchantRegistryContractClient<'a>, Address, Address) {
    // Using the updated register syntax
    let contract_id = env.register(MerchantRegistryContract, ());
    let client = MerchantRegistryContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let merchant = Address::generate(env);

    // Initialize the contract with the admin
    client.initialize(&admin);

    (client, admin, merchant)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    // Using the updated register syntax
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
    // Advance ledger time to test registration_date
    env.ledger().with_mut(|l| l.timestamp = 1000000);

    let name = String::from_str(&env, "Galaxy Tech Supplies");
    client.register_merchant(&admin, &merchant, &name);

    assert!(client.is_active(&merchant));

    // get_merchant_info automatically unwraps on success in the test client
    let info = client.get_merchant_info(&merchant);
    assert_eq!(info.name, name);
    assert_eq!(info.registration_date, 1000000);
    assert_eq!(info.active, true);
    assert_eq!(info.total_sales, 0);
    assert_eq!(client.get_merchant_count(), 1);
}

#[test]
fn test_duplicate_prevention() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let name = String::from_str(&env, "Stellar Books");
    client.register_merchant(&admin, &merchant, &name);

    // Registering the exact same address again should fail
    let res = client.try_register_merchant(&admin, &merchant, &name);
    assert!(res.is_err());
}

#[test]
#[should_panic(expected = "unauthorized: admin mismatch")]
fn test_admin_only_access() {
    let env = Env::default();
    let (client, _admin, merchant) = setup(&env);
    env.mock_all_auths();

    // Create a rogue admin address
    let fake_admin = Address::generate(&env);
    let name = String::from_str(&env, "Rogue Merchant");

    // This should panic due to require_admin auth
    client.register_merchant(&fake_admin, &merchant, &name);
}

#[test]
fn test_activation_and_deactivation() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let name = String::from_str(&env, "Nebula Cafe");
    client.register_merchant(&admin, &merchant, &name);

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
