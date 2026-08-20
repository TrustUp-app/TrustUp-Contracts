use crate::{default_parameters, ParametersContract, ParametersContractClient, ProtocolParameters};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};
/// Timelock used across governance tests: 1 day.
const TIMELOCK_SECS: u64 = 86_400;

fn setup() -> (Env, ParametersContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ParametersContract, ());
    let client = ParametersContractClient::new(&env, &contract_id);
    let client: ParametersContractClient<'static> = unsafe { core::mem::transmute(client) };
    let admin = Address::generate(&env);

    (env, client, admin)
}

#[test]
fn test_initialize_defaults() {
    let (_env, client, admin) = setup();
    client.initialize_defaults(&admin);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_parameters(), default_parameters());
}

#[test]
fn test_update_parameters() {
    let (_env, client, admin) = setup();
    client.initialize_defaults(&admin);

    let params = ProtocolParameters {
        min_guarantee_percent: 30,
        min_reputation_threshold: 70,
        full_repayment_reward: 12,
        default_penalty: 25,
        large_loan_threshold: 7_500,
        large_loan_default_penalty: 40,
        base_interest_bps: 900,
        grace_period_seconds: 86_400,
    };

    client.update_parameters(&admin, &params);
    assert_eq!(client.get_parameters(), params);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_update_parameters() {
    let (_env, client, admin) = setup();
    client.initialize_defaults(&admin);

    let intruder = Address::generate(&_env);
    let params = default_parameters();
    client.update_parameters(&intruder, &params);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_invalid_parameters_rejected() {
    let (_env, client, admin) = setup();

    let params = ProtocolParameters {
        min_guarantee_percent: 0,
        ..default_parameters()
    };

    client.initialize(&admin, &params);
}

// ─── governance: proposal / multi-sig / timelock / pause ──────────────────

/// Boots a contract already migrated to a 3-signer / 2-of-3 / 1-day-timelock
/// governance set, on top of the default parameters.
fn setup_governed() -> (Env, ParametersContractClient<'static>, Address, Address, Address, Address) {
    let (env, client, admin) = setup();
    client.initialize_defaults(&admin);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let s3 = Address::generate(&env);
    let signers = vec![&env, s1.clone(), s2.clone(), s3.clone()];

    client.migrate_to_multisig(&admin, &signers, &2, &TIMELOCK_SECS);
    (env, client, admin, s1, s2, s3)
}

#[test]
fn test_migrate_to_multisig() {
    let (_env, client, _admin, s1, s2, s3) = setup_governed();

    assert_eq!(client.get_signers(), vec![&_env, s1, s2, s3]);
    assert_eq!(client.get_threshold(), 2);
    assert_eq!(client.get_timelock(), TIMELOCK_SECS);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_migrate_twice_fails() {
    let (env, client, admin, ..) = setup_governed();
    let signers = vec![&env, admin.clone()];
    client.migrate_to_multisig(&admin, &signers, &1, &TIMELOCK_SECS);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_migrate_invalid_threshold_rejected() {
    let (env, client, admin) = setup();
    client.initialize_defaults(&admin);

    let signers = vec![&env, Address::generate(&env)];
    client.migrate_to_multisig(&admin, &signers, &2, &TIMELOCK_SECS);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_update_parameters_disabled_after_migration() {
    let (_env, client, admin, ..) = setup_governed();
    client.update_parameters(&admin, &default_parameters());
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_set_admin_disabled_after_migration() {
    let (env, client, admin, ..) = setup_governed();
    client.set_admin(&Address::generate(&env));
    let _ = admin;
}

#[test]
fn test_propose_approve_execute_parameters_happy_path() {
    let (env, client, _admin, s1, s2, _s3) = setup_governed();

    let params = ProtocolParameters {
        min_guarantee_percent: 30,
        ..default_parameters()
    };

    let id = client.propose_parameters(&s1, &params);
    client.approve_proposal(&s2, &id);

    env.ledger().with_mut(|li| li.timestamp += TIMELOCK_SECS);
    client.execute_proposal(&s1, &id);

    assert_eq!(client.get_parameters(), params);
    assert_eq!(client.get_proposal(&id).status, crate::ProposalStatus::Executed);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_execute_fails_insufficient_approvals() {
    let (env, client, _admin, s1, ..) = setup_governed();

    let id = client.propose_parameters(&s1, &default_parameters());
    env.ledger().with_mut(|li| li.timestamp += TIMELOCK_SECS);
    client.execute_proposal(&s1, &id);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_execute_fails_before_timelock_elapses() {
    let (_env, client, _admin, s1, s2, _s3) = setup_governed();

    let id = client.propose_parameters(&s1, &default_parameters());
    client.approve_proposal(&s2, &id);
    // No time advanced: threshold met but timelock has not elapsed yet.
    client.execute_proposal(&s1, &id);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_unauthorized_proposer_rejected() {
    let (env, client, ..) = setup_governed();
    let intruder = Address::generate(&env);
    client.propose_parameters(&intruder, &default_parameters());
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_unauthorized_approver_rejected() {
    let (env, client, _admin, s1, ..) = setup_governed();
    let id = client.propose_parameters(&s1, &default_parameters());

    let intruder = Address::generate(&env);
    client.approve_proposal(&intruder, &id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_double_approval_rejected() {
    let (_env, client, _admin, s1, ..) = setup_governed();
    let id = client.propose_parameters(&s1, &default_parameters());
    // s1 already approved implicitly as proposer.
    client.approve_proposal(&s1, &id);
}

#[test]
fn test_cancel_proposal_by_proposer() {
    let (_env, client, _admin, s1, ..) = setup_governed();
    let id = client.propose_parameters(&s1, &default_parameters());

    client.cancel_proposal(&s1, &id);
    assert_eq!(client.get_proposal(&id).status, crate::ProposalStatus::Cancelled);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_execute_finalized_proposal_fails() {
    let (_env, client, _admin, s1, ..) = setup_governed();
    let id = client.propose_parameters(&s1, &default_parameters());

    client.cancel_proposal(&s1, &id);
    client.execute_proposal(&s1, &id);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_get_unknown_proposal_fails() {
    let (_env, client, ..) = setup_governed();
    client.get_proposal(&999);
}

#[test]
fn test_pause_and_unpause_via_governance() {
    let (_env, client, _admin, s1, s2, _s3) = setup_governed();

    // Pauses skip the timelock: they take effect as soon as the threshold
    // of approvals is reached.
    let pause_id = client.propose_pause(&s1, &true);
    client.approve_proposal(&s2, &pause_id);
    client.execute_proposal(&s1, &pause_id);
    assert!(client.is_paused());

    let unpause_id = client.propose_pause(&s1, &false);
    client.approve_proposal(&s2, &unpause_id);
    client.execute_proposal(&s1, &unpause_id);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_pause_when_already_paused_rejected() {
    let (_env, client, _admin, s1, s2, _s3) = setup_governed();

    let pause_id = client.propose_pause(&s1, &true);
    client.approve_proposal(&s2, &pause_id);
    client.execute_proposal(&s1, &pause_id);

    client.propose_pause(&s1, &true);
}
