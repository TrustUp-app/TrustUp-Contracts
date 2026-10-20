#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, IntoVal, String, Symbol, TryFromVal,
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
fn test_admin_only_access() {
    let env = Env::default();
    let (client, _admin, merchant) = setup(&env);
    env.mock_all_auths();

    // Create a rogue admin address
    let fake_admin = Address::generate(&env);
    let name = String::from_str(&env, "Rogue Merchant");

    // Use try_register_merchant to catch the expected Unauthorized error
    let res = client.try_register_merchant(&fake_admin, &merchant, &name);
    assert!(res.is_err());
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

#[test]
fn test_set_merchant_status() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let name = String::from_str(&env, "Quasar Goods");
    client.register_merchant(&admin, &merchant, &name);

    // Merchant starts active
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
fn test_register_merchant_emits_registered_event() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let name = String::from_str(&env, "Event Merchant");
    client.register_merchant(&admin, &merchant, &name);

    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap();
    assert_eq!(contract_id, client.address);
    assert_eq!(
        topics,
        (Symbol::new(&env, "MERCHTREG"), merchant.clone()).into_val(&env)
    );
    assert_eq!(String::try_from_val(&env, &data).unwrap(), name);
}

#[test]
fn test_status_changes_emit_status_event() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    client.register_merchant(
        &admin,
        &merchant,
        &String::from_str(&env, "Status Merchant"),
    );

    client.deactivate_merchant(&admin, &merchant);
    let (_, topics, data) = env.events().all().last().unwrap();
    assert_eq!(
        topics,
        (Symbol::new(&env, "MERCHTSTATUS"), merchant.clone()).into_val(&env)
    );
    assert!(!bool::try_from_val(&env, &data).unwrap());

    client.activate_merchant(&admin, &merchant);
    let (_, _, data) = env.events().all().last().unwrap();
    assert!(bool::try_from_val(&env, &data).unwrap());
}

#[test]
fn test_failed_registration_emits_no_event() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let res = client.try_register_merchant(&admin, &merchant, &String::from_str(&env, ""));
    assert!(res.is_err());
    assert!(env.events().all().is_empty());
}

#[test]
fn test_name_length_bounds() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    // Empty name is rejected
    let res = client.try_register_merchant(&admin, &merchant, &String::from_str(&env, ""));
    assert_eq!(res, Err(Ok(Error::InvalidName)));

    // 65 characters is rejected
    let too_long = String::from_str(&env, &"a".repeat(65));
    let res = client.try_register_merchant(&admin, &merchant, &too_long);
    assert_eq!(res, Err(Ok(Error::InvalidName)));
    assert!(!client.is_active(&merchant));
    assert_eq!(client.get_merchant_count(), 0);

    // 64 characters is accepted and stored unchanged
    let max_len = String::from_str(&env, &"a".repeat(64));
    client.register_merchant(&admin, &merchant, &max_len);
    assert_eq!(client.get_merchant_info(&merchant).name, max_len);
}

#[test]
fn test_register_before_initialize_fails() {
    let env = Env::default();
    let contract_id = env.register(MerchantRegistryContract, ());
    let client = MerchantRegistryContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let res = client.try_register_merchant(&admin, &merchant, &String::from_str(&env, "Early"));
    assert_eq!(res, Err(Ok(Error::NotInitialized)));
}

#[test]
fn test_unknown_merchant_queries_and_status_changes() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    assert!(!client.is_active(&merchant));
    assert_eq!(
        client.try_get_merchant_info(&merchant),
        Err(Ok(Error::MerchantNotFound))
    );
    assert_eq!(
        client.try_deactivate_merchant(&admin, &merchant),
        Err(Ok(Error::MerchantNotFound))
    );
    assert_eq!(
        client.try_set_merchant_status(&admin, &merchant, &true),
        Err(Ok(Error::MerchantNotFound))
    );
}

#[test]
fn test_merchant_count_tracks_registrations() {
    let env = Env::default();
    let (client, admin, merchant) = setup(&env);
    env.mock_all_auths();

    let second = Address::generate(&env);
    client.register_merchant(&admin, &merchant, &String::from_str(&env, "First"));
    client.register_merchant(&admin, &second, &String::from_str(&env, "Second"));

    assert_eq!(client.get_merchant_count(), 2);
    assert_eq!(
        client.get_merchant_info(&second).name,
        String::from_str(&env, "Second")
    );
}
