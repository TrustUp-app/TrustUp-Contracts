#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

use crate::ReputationContract;
use crate::ReputationContractClient;

/// Test: Sets the contract admin
/// Verifies that an address can be assigned as the contract administrator.
/// Receives: Admin Address. Returns: void. Validates that the admin is stored correctly.
#[test]
fn it_sets_admin() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let retrieved_admin = client.get_admin();
    assert_eq!(retrieved_admin, admin);
}

/// Test: Gets the contract admin
/// Verifies that the current contract administrator can be queried.
/// Receives: nothing. Returns: Admin Address. Validates that it returns the correct address.
#[test]
fn it_gets_admin() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let retrieved = client.get_admin();
    assert_eq!(retrieved, admin);
}

/// Test: Assigns updater permissions
/// Verifies that the admin can grant updater permissions to an address.
/// Receives: Admin Address, Updater Address, bool allowed. Returns: void. Validates that the updater is authorized.
#[test]
fn it_sets_updater() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    assert_eq!(client.is_updater(&updater), true);
}

/// Test: Checks updater permissions
/// Verifies that it can be queried whether an address has updater permissions or not.
/// Receives: Address to check. Returns: bool (true if updater, false if not). Validates both cases.
#[test]
fn it_checks_updater() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    let non_updater = Address::generate(&env);
    
    client.set_updater(&admin, &updater, &true);
    
    assert_eq!(client.is_updater(&updater), true);
    assert_eq!(client.is_updater(&non_updater), false);
}

/// Test: Gets the reputation score
/// Verifies that a user's score can be queried. New users have a score of 0.
/// Receives: User Address. Returns: u32 (score 0-100). Validates initial read and after set.
#[test]
fn it_gets_score() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    let user = Address::generate(&env);
    
    // New user should have score 0
    assert_eq!(client.get_score(&user), 0);
    
    // Set score and verify
    client.set_score(&updater, &user, &50);
    assert_eq!(client.get_score(&user), 50);
}

/// Test: Increases the reputation score
/// Verifies that an authorized updater can increase a user's score.
/// Receives: Updater Address, User Address, u32 amount. Returns: void. Validates that the score increases correctly.
#[test]
fn it_increases_score() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    let user = Address::generate(&env);
    
    client.set_score(&updater, &user, &50);
    client.increase_score(&updater, &user, &20);
    
    assert_eq!(client.get_score(&user), 70);
}

/// Test: Decreases the reputation score
/// Verifies that an authorized updater can decrease a user's score.
/// Receives: Updater Address, User Address, u32 amount. Returns: void. Validates that the score decreases correctly.
#[test]
fn it_decreases_score() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    let user = Address::generate(&env);
    
    client.set_score(&updater, &user, &50);
    client.decrease_score(&updater, &user, &20);
    
    assert_eq!(client.get_score(&user), 30);
}

/// Test: Sets the score to a specific value
/// Verifies that an authorized updater can set a user's score to any valid value.
/// Receives: Updater Address, User Address, u32 new_score. Returns: void. Validates multiple score changes.
#[test]
fn it_sets_score() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    let user = Address::generate(&env);
    
    client.set_score(&updater, &user, &75);
    assert_eq!(client.get_score(&user), 75);
    
    client.set_score(&updater, &user, &25);
    assert_eq!(client.get_score(&user), 25);
}

/// Test: Prevents unauthorized updates
/// Verifies that only authorized updaters can modify scores. Users without permissions must fail.
/// Receives: Unauthorized Address attempting to update. Returns: panic with NotUpdater error (#2).
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn it_prevents_unauthorized_updates() {
    let env = Env::default();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.set_admin(&admin);
    
    let user = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    
    // Try to update score without being an updater (should panic)
    client.mock_all_auths().set_score(&unauthorized, &user, &50);
}

/// Test: Validates score bounds (0-100)
/// Verifies that scores cannot be set outside the valid range (0-100).
/// Receives: Invalid score (>100 or <0). Returns: panic with OutOfBounds error (#3). Protects data integrity.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn it_enforces_score_bounds() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    let user = Address::generate(&env);
    
    // Try to set score above maximum (should panic)
    client.set_score(&updater, &user, &101);
}

/// Test: Gets the contract version
/// Verifies that the contract returns its version identifier correctly.
/// Receives: nothing. Returns: Symbol "v1_0_0". Useful for verifying deployed version in production.
#[test]
fn it_gets_version() {
    let env = Env::default();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let version = client.get_version();
    assert_eq!(version, symbol_short!("v1_0_0"));
}

/// Test: Revokes updater access after removal
/// Verifies full lifecycle: grant -> modify -> revoke -> ensure NotUpdater error
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn it_revokes_updater_access_after_removal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    // Updater can update initially
    client.set_score(&updater, &user, &20);
    assert_eq!(client.get_score(&user), 20);

    // Revoke updater access
    client.set_updater(&admin, &updater, &false);
    assert_eq!(client.is_updater(&updater), false);

    // Former updater should no longer be able to increase the score (should panic NotUpdater)
    client.increase_score(&updater, &user, &5);
}

/// Test: Emitted event on updater removal
/// Verifies that UPDCHGD event can be emitted when removing updater
#[test]
fn it_emits_event_on_updater_removal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    // Grant updater
    client.set_updater(&admin, &updater, &true);
    assert_eq!(client.is_updater(&updater), true);

    // Revoke updater (this should emit UPDCHGD event with false)
    client.set_updater(&admin, &updater, &false);
    assert_eq!(client.is_updater(&updater), false);

    // If we reached here without panic, event emission didn't crash
}

/// Test: Removing a non-existent updater should not panic and should be handled gracefully
#[test]
fn it_handles_removing_non_existent_updater() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let never_added = Address::generate(&env);

    // Removing a never-added updater should not panic
    client.set_updater(&admin, &never_added, &false);
    assert_eq!(client.is_updater(&never_added), false);
}

/// Overflow/Underflow tests
/// Verifies contract returns correct errors for arithmetic edge cases
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn it_prevents_overflow_on_increase() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    client.set_score(&updater, &user, &80);
    // increase by 50 would overflow beyond 100
    client.increase_score(&updater, &user, &50);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn it_prevents_overflow_at_max() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    client.set_score(&updater, &user, &100);
    // increase by 1 when at max should overflow
    client.increase_score(&updater, &user, &1);
}

#[test]
fn it_allows_increase_up_to_max() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    client.set_score(&updater, &user, &80);
    client.increase_score(&updater, &user, &20);
    assert_eq!(client.get_score(&user), 100);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn it_prevents_underflow_on_decrease() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    client.set_score(&updater, &user, &30);
    // decrease by 50 would underflow below 0
    client.decrease_score(&updater, &user, &50);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn it_prevents_underflow_at_min() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    client.set_score(&updater, &user, &0);
    // decrease by 1 at zero should underflow
    client.decrease_score(&updater, &user, &1);
}

#[test]
fn it_allows_decrease_down_to_min() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);

    let user = Address::generate(&env);

    client.set_score(&updater, &user, &30);
    client.decrease_score(&updater, &user, &30);
    assert_eq!(client.get_score(&user), 0);
}

/// Test: Removing one updater doesn't affect other updaters
#[test]
fn it_removing_one_updater_does_not_affect_others() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let updater1 = Address::generate(&env);
    let updater2 = Address::generate(&env);
    client.set_updater(&admin, &updater1, &true);
    client.set_updater(&admin, &updater2, &true);

    let user = Address::generate(&env);

    // updater1 increases score
    client.set_score(&updater1, &user, &10);
    client.increase_score(&updater1, &user, &5);
    assert_eq!(client.get_score(&user), 15);

    // Revoke updater1
    client.set_updater(&admin, &updater1, &false);
    assert_eq!(client.is_updater(&updater1), false);
    assert_eq!(client.is_updater(&updater2), true);

    // updater2 should still be able to update
    client.increase_score(&updater2, &user, &5);
    assert_eq!(client.get_score(&user), 20);
}

