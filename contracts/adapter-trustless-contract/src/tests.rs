use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Events, Ledger},
    Address, Env, IntoVal, Symbol, Val, Vec,
};

use crate::AdapterTrustlessContract;
use crate::AdapterTrustlessContractClient;

// A minimal target contract used to exercise `execute_action`'s cross-contract
// invocation. Mirrors the kind of admin-only setter the adapter is meant to guard.
#[contract]
pub struct MockTarget;

#[contractimpl]
impl MockTarget {
    pub fn set_value(env: Env, value: u32) -> u32 {
        env.storage()
            .instance()
            .set(&symbol_short!("VALUE"), &value);
        value
    }

    pub fn get_value(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("VALUE"))
            .unwrap_or(0)
    }
}

fn setup(env: &Env) -> (AdapterTrustlessContractClient<'_>, Vec<Address>, Address) {
    let contract_id = env.register(AdapterTrustlessContract, ());
    let client = AdapterTrustlessContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    (client, Vec::new(env), admin)
}

fn set_value_call(env: &Env, target: &Address, value: u32) -> (Address, Symbol, Vec<Val>) {
    let mut args = Vec::new(env);
    args.push_back(value.into_val(env));
    (target.clone(), symbol_short!("set_value"), args)
}

/// Test: Initializes the adapter with signers, threshold and timelock
#[test]
fn it_initializes() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initialize(&admin, &signers, &2, &1000);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_signers(), signers);
    assert_eq!(client.get_threshold(), 2);
    assert_eq!(client.get_timelock(), 1000);
}

/// Test: Prevents double initialization
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn it_prevents_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);

    client.initialize(&admin, &signers, &1, &0);
    client.initialize(&admin, &signers, &1, &0);
}

/// Test: Rejects an empty signer set
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn it_rejects_empty_signers() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, signers, admin) = setup(&env);
    client.initialize(&admin, &signers, &1, &0);
}

/// Test: Rejects a threshold greater than the number of signers
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn it_rejects_threshold_above_signer_count() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);

    client.initialize(&admin, &signers, &2, &0);
}

/// Test: A signer can propose an action, which auto-approves from the proposer
#[test]
fn it_proposes_action_with_proposer_auto_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);
    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    client.initialize(&admin, &signers, &1, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);

    let action_id = client.propose_action(&signer, &target, &function, &args);
    assert_eq!(action_id, 1);

    let action = client.get_action(&action_id);
    assert_eq!(action.approvals.len(), 1);
    assert_eq!(action.approvals.get(0).unwrap(), signer);
    assert!(!action.executed);
    assert!(!action.canceled);
}

/// Test: Only an authorized signer can propose an action
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn it_rejects_proposal_from_non_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);
    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);
    client.initialize(&admin, &signers, &1, &0);

    let outsider = Address::generate(&env);
    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 1);

    client.propose_action(&outsider, &target, &function, &args);
}

/// Test: Executes an action once threshold and timelock are satisfied
#[test]
fn it_executes_action_after_threshold_and_timelock() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initialize(&admin, &signers, &2, &1000);

    let target_id = env.register(MockTarget, ());
    let target_client = MockTargetClient::new(&env, &target_id);
    assert_eq!(target_client.get_value(), 0);

    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer1, &target, &function, &args);

    client.approve_action(&signer2, &action_id);

    env.ledger().with_mut(|li| li.timestamp += 1000);

    let caller = Address::generate(&env);
    client.execute_action(&caller, &action_id);

    assert_eq!(target_client.get_value(), 42);

    let action = client.get_action(&action_id);
    assert!(action.executed);
}

/// Test: Cannot execute before the approval threshold is met
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn it_prevents_execution_below_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initialize(&admin, &signers, &2, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer1, &target, &function, &args);

    let caller = Address::generate(&env);
    client.execute_action(&caller, &action_id);
}

/// Test: Cannot execute before the timelock has elapsed
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn it_prevents_execution_before_timelock_elapses() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    client.initialize(&admin, &signers, &1, &1000);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer, &target, &function, &args);

    let caller = Address::generate(&env);
    client.execute_action(&caller, &action_id);
}

/// Test: Cannot execute the same action twice
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn it_prevents_double_execution() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    client.initialize(&admin, &signers, &1, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer, &target, &function, &args);

    let caller = Address::generate(&env);
    client.execute_action(&caller, &action_id);
    client.execute_action(&caller, &action_id);
}

/// Test: A signer cannot approve the same action twice
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn it_prevents_double_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());
    client.initialize(&admin, &signers, &2, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer1, &target, &function, &args);

    client.approve_action(&signer1, &action_id);
}

/// Test: A signer can revoke their approval before execution
#[test]
fn it_revokes_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());
    client.initialize(&admin, &signers, &2, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer1, &target, &function, &args);

    client.approve_action(&signer2, &action_id);
    assert!(client.is_approved(&action_id));

    client.revoke_approval(&signer2, &action_id);
    assert!(!client.is_approved(&action_id));

    let action = client.get_action(&action_id);
    assert_eq!(action.approvals.len(), 1);
}

/// Test: Admin can cancel a pending action, blocking execution
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn it_cancels_action() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    client.initialize(&admin, &signers, &1, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer, &target, &function, &args);

    client.cancel_action(&admin, &action_id);

    let caller = Address::generate(&env);
    client.execute_action(&caller, &action_id);
}

/// Test: Only the admin can cancel an action
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn it_rejects_cancel_from_non_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    client.initialize(&admin, &signers, &1, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 42);
    let action_id = client.propose_action(&signer, &target, &function, &args);

    let outsider = Address::generate(&env);
    client.cancel_action(&outsider, &action_id);
}

/// Test: Admin can add and remove signers
#[test]
fn it_adds_and_removes_signers() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    client.initialize(&admin, &signers, &1, &0);

    let signer2 = Address::generate(&env);
    client.add_signer(&admin, &signer2);
    assert_eq!(client.get_signers().len(), 2);

    client.remove_signer(&admin, &signer1);
    assert_eq!(client.get_signers().len(), 1);
    assert!(!client.get_signers().contains(&signer1));
}

/// Test: Cannot remove a signer if it would drop below the current threshold
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn it_prevents_removing_signer_below_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());
    client.initialize(&admin, &signers, &2, &0);

    client.remove_signer(&admin, &signer1);
}

/// Test: Admin can raise or lower the threshold within signer bounds
#[test]
fn it_changes_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1);
    signers.push_back(signer2);
    client.initialize(&admin, &signers, &1, &0);

    client.set_threshold(&admin, &2);
    assert_eq!(client.get_threshold(), 2);
}

/// Test: Admin can change the timelock delay
#[test]
fn it_changes_timelock() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);
    client.initialize(&admin, &signers, &1, &0);

    client.set_timelock(&admin, &500);
    assert_eq!(client.get_timelock(), 500);
}

/// Test: Admin can transfer admin rights
#[test]
fn it_transfers_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);
    client.initialize(&admin, &signers, &1, &0);

    let new_admin = Address::generate(&env);
    client.set_admin(&admin, &new_admin);
    assert_eq!(client.get_admin(), new_admin);
}

/// Test: Fetching an unknown action id panics with ActionNotFound
#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn it_fails_on_unknown_action() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);
    client.initialize(&admin, &signers, &1, &0);

    client.get_action(&99);
}

/// Test: Emits ACTNEXEC event on execution
#[test]
fn it_emits_action_executed_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _unused, admin) = setup(&env);

    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    client.initialize(&admin, &signers, &1, &0);

    let target_id = env.register(MockTarget, ());
    let (target, function, args) = set_value_call(&env, &target_id, 7);
    let action_id = client.propose_action(&signer, &target, &function, &args);

    let caller = Address::generate(&env);
    client.execute_action(&caller, &action_id);

    let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();

    let mut found = false;
    for event in events.iter() {
        let topics = event.1.clone();
        let event_type: Symbol = topics.get(0).unwrap().into_val(&env);

        if event_type == symbol_short!("ACTNEXEC") {
            found = true;
            break;
        }
    }

    assert!(found, "ACTNEXEC event not found");
}
