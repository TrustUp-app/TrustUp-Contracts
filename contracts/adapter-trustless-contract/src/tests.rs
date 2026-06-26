#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

use crate::{AdapterTrustlessContract, AdapterTrustlessContractClient, EscrowStatus};

// ── Test helpers ─────────────────────────────────────────────────────────────

struct TestCtx {
    env: Env,
    client: AdapterTrustlessContractClient<'static>,
    admin: Address,
    creditline: Address,
    token: Address,
}

fn setup() -> TestCtx {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creditline = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    let contract_id = env.register(AdapterTrustlessContract, ());
    let client = AdapterTrustlessContractClient::new(&env, &contract_id);

    client.initialize(&admin, &creditline, &token);

    TestCtx { env, client, admin, creditline, token }
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    // StellarAssetClient mint requires the token admin auth, covered by mock_all_auths
    StellarAssetClient::new(env, token).mint(to, &amount);
}

// ── Initialisation ────────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin_and_creditline() {
    let ctx = setup();
    assert_eq!(ctx.client.get_admin(), ctx.admin);
    assert_eq!(ctx.client.get_creditline(), ctx.creditline);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_initialize_twice_panics() {
    let ctx = setup();
    ctx.client.initialize(&ctx.admin, &ctx.creditline, &ctx.token);
}

// ── Admin operations ──────────────────────────────────────────────────────────

#[test]
fn test_set_admin_transfers_admin() {
    let ctx = setup();
    let new_admin = Address::generate(&ctx.env);
    ctx.client.set_admin(&new_admin);
    assert_eq!(ctx.client.get_admin(), new_admin);
}

#[test]
fn test_set_creditline_updates_creditline() {
    let ctx = setup();
    let new_cl = Address::generate(&ctx.env);
    ctx.client.set_creditline(&ctx.admin, &new_cl);
    assert_eq!(ctx.client.get_creditline(), new_cl);
}

#[test]
#[should_panic(expected = "NotAdmin")]
fn test_set_creditline_rejects_non_admin() {
    let ctx = setup();
    let attacker = Address::generate(&ctx.env);
    let new_cl = Address::generate(&ctx.env);
    ctx.client.set_creditline(&attacker, &new_cl);
}

// ── lock_guarantee ────────────────────────────────────────────────────────────

#[test]
fn test_lock_guarantee_stores_entry() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);

    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);

    let entry = ctx.client.get_escrow(&1u64);
    assert_eq!(entry.borrower, borrower);
    assert_eq!(entry.amount, 200);
    assert_eq!(entry.status, EscrowStatus::Locked);
}

#[test]
#[should_panic(expected = "NotCreditLine")]
fn test_lock_guarantee_rejects_non_creditline() {
    let ctx = setup();
    let attacker = Address::generate(&ctx.env);
    let borrower = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);
    ctx.client.lock_guarantee(&attacker, &1u64, &borrower, &200i128);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_lock_guarantee_rejects_zero_amount() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &0i128);
}

#[test]
#[should_panic(expected = "InvalidStatus")]
fn test_lock_guarantee_rejects_duplicate_loan_id() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 400);
    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
}

// ── release_guarantee ─────────────────────────────────────────────────────────

#[test]
fn test_release_guarantee_returns_funds_to_borrower() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);

    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
    ctx.client.release_guarantee(&ctx.creditline, &1u64);

    let entry = ctx.client.get_escrow(&1u64);
    assert_eq!(entry.status, EscrowStatus::Released);

    let balance = TokenClient::new(&ctx.env, &ctx.token).balance(&borrower);
    assert_eq!(balance, 200);
}

#[test]
#[should_panic(expected = "InvalidStatus")]
fn test_release_guarantee_twice_panics() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);
    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
    ctx.client.release_guarantee(&ctx.creditline, &1u64);
    ctx.client.release_guarantee(&ctx.creditline, &1u64);
}

#[test]
#[should_panic(expected = "EscrowNotFound")]
fn test_release_guarantee_unknown_loan_panics() {
    let ctx = setup();
    ctx.client.release_guarantee(&ctx.creditline, &99u64);
}

// ── seize_guarantee ───────────────────────────────────────────────────────────

#[test]
fn test_seize_guarantee_forwards_funds_to_pool() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    let pool = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);

    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
    ctx.client.seize_guarantee(&ctx.creditline, &1u64, &pool);

    let entry = ctx.client.get_escrow(&1u64);
    assert_eq!(entry.status, EscrowStatus::Seized);

    let pool_balance = TokenClient::new(&ctx.env, &ctx.token).balance(&pool);
    assert_eq!(pool_balance, 200);
}

#[test]
#[should_panic(expected = "InvalidStatus")]
fn test_seize_after_release_panics() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    let pool = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);
    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
    ctx.client.release_guarantee(&ctx.creditline, &1u64);
    ctx.client.seize_guarantee(&ctx.creditline, &1u64, &pool);
}

#[test]
#[should_panic(expected = "NotCreditLine")]
fn test_seize_guarantee_rejects_non_creditline() {
    let ctx = setup();
    let borrower = Address::generate(&ctx.env);
    let pool = Address::generate(&ctx.env);
    let attacker = Address::generate(&ctx.env);
    mint(&ctx.env, &ctx.token, &borrower, 200);
    ctx.client.lock_guarantee(&ctx.creditline, &1u64, &borrower, &200i128);
    ctx.client.seize_guarantee(&attacker, &1u64, &pool);
}
