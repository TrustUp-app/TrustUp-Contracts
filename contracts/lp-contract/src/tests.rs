#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env};

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(LiquidityPoolContract, ());

    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let client = LiquidityPoolContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_id);

    (env, contract_id, token_id, admin)
}

fn mint_tokens(env: &Env, token_id: &Address, _admin: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token_id).mint(to, &amount);
}

#[test]
fn test_first_deposit_one_to_one() {
    let (env, contract_id, token_id, admin) = setup();
    let client = LiquidityPoolContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    mint_tokens(&env, &token_id, &admin, &provider, 1_000);

    let shares = client.deposit(&provider, &1_000);

    assert_eq!(shares, 1_000, "first deposit must be 1:1");
    assert_eq!(client.get_lp_shares(&provider), 1_000);

    let stats = client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 1_000);
    assert_eq!(stats.total_shares, 1_000);
}

/// Subsequent deposits must issue shares proportionally to the current pool value.
/// Formula: shares = (amount × total_shares) / total_pool_value
#[test]
fn test_subsequent_deposit_proportional() {
    let (env, contract_id, token_id, admin) = setup();
    let client = LiquidityPoolContractClient::new(&env, &contract_id);

    let provider_a = Address::generate(&env);
    let provider_b = Address::generate(&env);

    mint_tokens(&env, &token_id, &admin, &provider_a, 1_000);
    mint_tokens(&env, &token_id, &admin, &provider_b, 500);

    // First deposit: 1000 tokens → 1000 shares
    client.deposit(&provider_a, &1_000);

    // Second deposit: 500 tokens into pool of 1000 tokens / 1000 shares
    // Expected shares = (500 × 1000) / 1000 = 500
    let shares_b = client.deposit(&provider_b, &500);

    assert_eq!(shares_b, 500, "subsequent deposit must be proportional");
    assert_eq!(client.get_lp_shares(&provider_b), 500);

    let stats = client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 1_500);
    assert_eq!(stats.total_shares, 1_500);
}

/// Share calculation must stay accurate across multiple deposits with varied amounts.
#[test]
fn test_share_calculation_accuracy() {
    let (env, contract_id, token_id, admin) = setup();
    let client = LiquidityPoolContractClient::new(&env, &contract_id);

    let provider_a = Address::generate(&env);
    let provider_b = Address::generate(&env);
    let provider_c = Address::generate(&env);

    mint_tokens(&env, &token_id, &admin, &provider_a, 2_000);
    mint_tokens(&env, &token_id, &admin, &provider_b, 1_000);
    mint_tokens(&env, &token_id, &admin, &provider_c, 3_000);

    // Deposit 1: 2000 tokens → 2000 shares (1:1)
    client.deposit(&provider_a, &2_000);

    // Deposit 2: 1000 tokens => (1000 × 2000) / 2000 = 1000 shares
    let shares_b = client.deposit(&provider_b, &1_000);
    assert_eq!(shares_b, 1_000);

    // Deposit 3: 3000 tokens => (3000 × 3000) / 3000 = 3000 shares
    let shares_c = client.deposit(&provider_c, &3_000);
    assert_eq!(shares_c, 3_000);

    // Total: 6000 tokens, 6000 shares => each share worth exactly 1 token
    let stats = client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 6_000);
    assert_eq!(stats.total_shares, 6_000);
}

/// Depositing zero (or negative) tokens must be rejected.
#[test]
#[should_panic]
fn test_zero_deposit_rejected() {
    let (env, contract_id, _token_id, _admin) = setup();
    let client = LiquidityPoolContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);

    // Zero deposit must panic with InvalidAmount
    client.deposit(&provider, &0);
}
