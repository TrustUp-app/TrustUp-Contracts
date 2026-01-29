#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::{Address as _, Events}, Address, Env, IntoVal, Symbol, Vec, Val};

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

/// Test: Emits SCORECHGD event on score increase
/// Verifies that increasing a user's score emits the correct event with (user, old_score, new_score, "increase") data.
#[test]
fn it_emits_score_changed_event_on_increase() {
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
    
    // Increase score
    client.increase_score(&updater, &user, &20);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the SCORECHGD event (should be the last one)
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("SCORECHGD") {
            found_event = true;
            
            // Verify user address (second topic)
            let event_user: Address = topics.get(1).unwrap().into_val(&env);
            assert_eq!(event_user, user);
            
            // Verify data (old_score, new_score, reason) - data is a tuple
            let data_tuple: (u32, u32, Symbol) = event.2.into_val(&env);
            let (old_score, new_score, reason) = data_tuple;
            
            assert_eq!(old_score, 50);
            assert_eq!(new_score, 70);
            assert_eq!(reason, symbol_short!("increase"));
            break;
        }
    }
    
    assert!(found_event, "SCORECHGD event not found");
}

/// Test: Emits SCORECHGD event on score decrease
/// Verifies that decreasing a user's score emits the correct event with (user, old_score, new_score, "decrease") data.
#[test]
fn it_emits_score_changed_event_on_decrease() {
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
    
    // Decrease score
    client.decrease_score(&updater, &user, &20);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the SCORECHGD event
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("SCORECHGD") {
            // Check if this is the decrease event (new_score should be 30)
            let data_tuple: (u32, u32, Symbol) = event.2.into_val(&env);
            let (_, _new_score, reason) = data_tuple;
            
            if reason == symbol_short!("decrease") {
                found_event = true;
                
                // Verify user address (second topic)
                let event_user: Address = topics.get(1).unwrap().into_val(&env);
                assert_eq!(event_user, user);
                
                let (old_score, new_score, _) = data_tuple;
                assert_eq!(old_score, 50);
                assert_eq!(new_score, 30);
                break;
            }
        }
    }
    
    assert!(found_event, "SCORECHGD event with 'decrease' reason not found");
}

/// Test: Emits SCORECHGD event on score set
/// Verifies that setting a user's score emits the correct event with (user, old_score, new_score, "set") data.
#[test]
fn it_emits_score_changed_event_on_set() {
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
    
    // Set score to new value
    client.set_score(&updater, &user, &75);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the SCORECHGD event with "set" reason
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("SCORECHGD") {
            let data_tuple: (u32, u32, Symbol) = event.2.into_val(&env);
            let (_, _, reason) = data_tuple;
            
            if reason == symbol_short!("set") {
                found_event = true;
                
                // Verify user address (second topic)
                let event_user: Address = topics.get(1).unwrap().into_val(&env);
                assert_eq!(event_user, user);
                
                let (old_score, new_score, _) = data_tuple;
                assert_eq!(old_score, 50);
                assert_eq!(new_score, 75);
                break;
            }
        }
    }
    
    assert!(found_event, "SCORECHGD event with 'set' reason not found");
}

/// Test: Emits UPDCHGD event on updater grant
/// Verifies that granting updater permission emits the correct event with (updater, true) data.
#[test]
fn it_emits_updater_changed_event_on_grant() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    
    // Grant updater permission
    client.set_updater(&admin, &updater, &true);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the UPDCHGD event with allowed=true
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("UPDCHGD") {
            let allowed: bool = event.2.into_val(&env);
            
            if allowed {
                found_event = true;
                
                // Verify updater address (second topic)
                let event_updater: Address = topics.get(1).unwrap().into_val(&env);
                assert_eq!(event_updater, updater);
                break;
            }
        }
    }
    
    assert!(found_event, "UPDCHGD event with allowed=true not found");
}

/// Test: Emits UPDCHGD event on updater revoke
/// Verifies that revoking updater permission emits the correct event with (updater, false) data.
#[test]
fn it_emits_updater_changed_event_on_revoke() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let updater = Address::generate(&env);
    client.set_updater(&admin, &updater, &true);
    
    // Revoke updater permission
    client.set_updater(&admin, &updater, &false);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the UPDCHGD event with allowed=false
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("UPDCHGD") {
            let allowed: bool = event.2.into_val(&env);
            
            if !allowed {
                found_event = true;
                
                // Verify updater address (second topic)
                let event_updater: Address = topics.get(1).unwrap().into_val(&env);
                assert_eq!(event_updater, updater);
                break;
            }
        }
    }
    
    assert!(found_event, "UPDCHGD event with allowed=false not found");
}

/// Test: Emits ADMINCHGD event on admin change
/// Verifies that changing the admin emits the correct event with (old_admin, new_admin) data.
#[test]
fn it_emits_admin_changed_event() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    
    let new_admin = Address::generate(&env);
    
    // Change admin
    client.set_admin(&new_admin);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the ADMINCHGD event where old_admin != new_admin
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("ADMINCHGD") {
            let data_tuple: (Address, Address) = event.2.into_val(&env);
            let (old_admin, new_admin_event) = data_tuple;
            
            // This should be the admin change event (not initial setup)
            if old_admin != new_admin_event {
                found_event = true;
                assert_eq!(old_admin, admin);
                assert_eq!(new_admin_event, new_admin);
                break;
            }
        }
    }
    
    assert!(found_event, "ADMINCHGD event for admin change not found");
}

/// Test: Emits ADMINCHGD event on initial admin setup
/// Verifies that setting the admin for the first time emits the event with (dummy_address, new_admin) data.
/// Note: The contract uses the new_admin as both old and new during initial setup.
#[test]
fn it_emits_admin_changed_event_on_initial_setup() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    
    // Set admin for the first time
    client.set_admin(&admin);
    
    // Verify event was emitted
    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    
    // Find the ADMINCHGD event
    let mut found_event = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);
        
        if event_type == symbol_short!("ADMINCHGD") {
            found_event = true;
            
            // Verify data (old_admin, new_admin) - data is a tuple
            let data_tuple: (Address, Address) = event.2.into_val(&env);
            let (old_admin, new_admin_event) = data_tuple;
            
            // During initial setup, both old and new admin are set to the same address (the new admin)
            assert_eq!(old_admin, admin);
            assert_eq!(new_admin_event, admin);
            break;
        }
    }
    
    assert!(found_event, "ADMINCHGD event not found");
}
