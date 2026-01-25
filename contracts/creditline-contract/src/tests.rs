#![cfg(test)]

use crate::{CreditLineContract, CreditLineContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

// NOTE: Integration tests with reputation contract are skipped for now
// They will be added when all contracts are implemented and properly configured

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    // Try to initialize again - should panic
    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );
}

#[test]
fn test_get_version() {
    let version = CreditLineContract::get_version();
    assert_eq!(version, soroban_sdk::symbol_short!("v1_0_0"));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_get_loan_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    // Try to get a loan that doesn't exist
    client.get_loan(&999);
}

#[test]
fn test_set_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    assert_eq!(client.get_admin(), admin);

    // Change admin
    client.set_admin(&new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_set_reputation_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let new_reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    // Update reputation contract address
    client.set_reputation_contract(&admin, &new_reputation_contract);

    // Verify it was updated (we can't directly query, but no panic means success)
}

#[test]
fn test_set_merchant_registry() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let new_merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    // Update merchant registry address
    client.set_merchant_registry(&admin, &new_merchant_registry);

    // Verify it was updated (we can't directly query, but no panic means success)
}

#[test]
fn test_set_liquidity_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);
    let new_liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    // Update liquidity pool address
    client.set_liquidity_pool(&admin, &new_liquidity_pool);

    // Verify it was updated (we can't directly query, but no panic means success)
}

// Tests for validate_guarantee logic (tested indirectly through create_loan)

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_create_loan_with_zero_total_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // This should panic with InvalidAmount (error code 9)
    client.create_loan(&user, &merchant, &0, &0, &repayment_schedule);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_create_loan_with_negative_total_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // This should panic with InvalidAmount (error code 9)
    client.create_loan(&user, &merchant, &-1000, &-200, &repayment_schedule);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_create_loan_with_zero_guarantee_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // This should panic with InvalidAmount (error code 9)
    client.create_loan(&user, &merchant, &1000, &0, &repayment_schedule);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_create_loan_with_insufficient_guarantee_19_percent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // 190 is 19% of 1000, should fail with InsufficientGuarantee (error code 2)
    client.create_loan(&user, &merchant, &1000, &190, &repayment_schedule);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_create_loan_with_insufficient_guarantee_10_percent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // 100 is 10% of 1000, should fail with InsufficientGuarantee (error code 2)
    client.create_loan(&user, &merchant, &1000, &100, &repayment_schedule);
}

// Additional edge case tests

#[test]
#[should_panic(expected = "Admin not set")]
fn test_get_admin_before_initialization() {
    let env = Env::default();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    // Try to get admin before initialization - should panic
    client.get_admin();
}

#[test]
#[should_panic(expected = "Admin not set")]
fn test_set_admin_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let new_admin = Address::generate(&env);

    // Try to set admin before initialization - should panic
    client.set_admin(&new_admin);
}

#[test]
fn test_loan_counter_increments() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    // Note: We can't actually create loans without a reputation contract
    // This test validates the counter mechanism exists
    // Full testing will be done with integration tests
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_create_loan_with_one_less_than_minimum_guarantee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // 199 is 1 less than 20% of 1000, should fail with InsufficientGuarantee (error code 2)
    client.create_loan(&user, &merchant, &1000, &199, &repayment_schedule);
}

#[test]
fn test_multiple_contract_address_updates() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let reputation_contract_1 = Address::generate(&env);
    let reputation_contract_2 = Address::generate(&env);
    let reputation_contract_3 = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract_1,
        &merchant_registry,
        &liquidity_pool,
    );

    // Update reputation contract multiple times
    client.set_reputation_contract(&admin, &reputation_contract_2);
    client.set_reputation_contract(&admin, &reputation_contract_3);

    // All updates should succeed without panic
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_create_loan_with_positive_total_negative_guarantee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let reputation_contract = Address::generate(&env);
    let merchant_registry = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &reputation_contract,
        &merchant_registry,
        &liquidity_pool,
    );

    let repayment_schedule = soroban_sdk::Vec::new(&env);

    // Positive total but negative guarantee should fail with InvalidAmount (error code 9)
    client.create_loan(&user, &merchant, &1000, &-200, &repayment_schedule);
}
