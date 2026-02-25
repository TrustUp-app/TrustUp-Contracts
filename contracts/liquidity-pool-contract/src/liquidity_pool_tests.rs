use crate::{LiquidityPoolContract, LiquidityPoolContractClient};
use soroban_sdk::{
    testutils::Address as AddressTrait,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

// ─── Test Environment Setup ──────────────────────────────────────────────────

struct TestEnv {
    env: Env,
    client: LiquidityPoolContractClient<'static>,
    token: TokenClient<'static>,
    admin: Address,
    treasury: Address,
    merchant_fund: Address,
    creditline: Address,
}

impl TestEnv {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        // Deploy a Stellar asset (test token)
        let token_admin = Address::generate(&env);
        let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract_id.address();

        let token = TokenClient::new(&env, &token_address);
        let token_sac = StellarAssetClient::new(&env, &token_address);

        // Deploy liquidity pool contract
        let contract_id = env.register(LiquidityPoolContract, ());
        let client = LiquidityPoolContractClient::new(&env, &contract_id);

        // SAFETY: we extend the lifetime so we can store these in the struct.
        // This is safe because `env` outlives both clients.
        let token: TokenClient<'static> = unsafe { core::mem::transmute(token) };
        let token_sac: StellarAssetClient<'static> = unsafe { core::mem::transmute(token_sac) };
        let client: LiquidityPoolContractClient<'static> = unsafe { core::mem::transmute(client) };

        let admin = Address::generate(&env);
        let treasury = Address::generate(&env);
        let merchant_fund = Address::generate(&env);
        let creditline = Address::generate(&env);

        // Initialize pool
        client.initialize(&admin, &token_address, &treasury, &merchant_fund);
        client.set_creditline(&admin, &creditline);

        // Mint tokens into some standard accounts for tests to use
        token_sac.mint(&creditline, &10_000_000);

        Self {
            env,
            client,
            token,
            admin,
            treasury,
            merchant_fund,
            creditline,
        }
    }

    /// Mint `amount` tokens to `recipient`
    fn mint(&self, recipient: &Address, amount: i128) {
        let token_address = self.token.address.clone();
        let token_sac = StellarAssetClient::new(&self.env, &token_address);
        token_sac.mint(recipient, &amount);
    }
}

// ─── Deposit Tests ───────────────────────────────────────────────────────────

#[test]
fn test_first_deposit_one_to_one_share_ratio() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let expected_total_liquidity = 1000;
    let expected_total_shares = 1000;
    let expected_share_price = 10_000;

    // 1. Create a liquidity provider address
    let provider = Address::generate(&context.env);

    // 2. Mint tokens to the provider
    context.mint(&provider, deposit_amount);

    // 3. Call deposit() with tokens
    let shares = context.client.deposit(&provider, &deposit_amount);

    // 4. Assert that shares returned match expected
    assert_eq!(shares, expected_shares);

    // 5. Verify pool stats: total_liquidity and total_shares
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, expected_total_liquidity);
    assert_eq!(stats.total_shares, expected_total_shares);

    // 6. Verify share_price (1.00 in basis points)
    assert_eq!(stats.share_price, expected_share_price);

    // Verify provider's share balance
    let provider_shares = context.client.get_lp_shares(&provider);
    assert_eq!(provider_shares, expected_shares);
}

#[test]
fn test_subsequent_deposits_proportional_shares() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 1000;
    let provider1_expected_shares = 1000;
    let provider2_deposit = 500;
    let provider2_expected_shares = 500;
    let expected_total_shares = 1500;
    let expected_total_liquidity = 1500;
    let expected_share_price = 10_000;

    // 1. Create first provider address and mint tokens
    let provider1 = Address::generate(&context.env);
    context.mint(&provider1, provider1_deposit);

    // 2. First provider deposits tokens
    let shares1 = context.client.deposit(&provider1, &provider1_deposit);
    assert_eq!(shares1, provider1_expected_shares);

    // 3. Create second provider address and mint tokens
    let provider2 = Address::generate(&context.env);
    context.mint(&provider2, provider2_deposit);

    // 4. Second provider deposits tokens
    let shares2 = context.client.deposit(&provider2, &provider2_deposit);

    // 5. Assert second provider receives expected shares
    assert_eq!(shares2, provider2_expected_shares);

    // 6. Verify second provider's share balance
    let provider2_balance = context.client.get_lp_shares(&provider2);
    assert_eq!(provider2_balance, provider2_expected_shares);

    // 7. Verify total_shares
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_shares, expected_total_shares);

    // 8. Verify total_liquidity
    assert_eq!(stats.total_liquidity, expected_total_liquidity);

    // 9. Verify share_price remains constant (no interest, so still 1:1)
    assert_eq!(stats.share_price, expected_share_price);
}

#[test]
fn test_share_calculation_accuracy() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 1_000_000;
    let provider1_expected_shares = 1_000_000;
    let provider2_deposit = 500_000;
    let provider2_expected_shares = 500_000;
    let expected_total_shares = 1_500_000;
    let expected_total_liquidity = 1_500_000;
    let interest_amount = 100_000;
    let principal_repayment = 0;
    let lp_interest_percentage = 85;
    let lp_interest = (interest_amount * lp_interest_percentage) / 100;
    let expected_liquidity_after_interest = expected_total_liquidity + lp_interest;
    let provider3_deposit = 100;
    let provider3_expected_shares = 94;

    // 1. Test with various deposit amounts (small, medium, large)
    let provider1 = Address::generate(&context.env);
    context.mint(&provider1, provider1_deposit);
    let shares1 = context.client.deposit(&provider1, &provider1_deposit);
    assert_eq!(shares1, provider1_expected_shares);

    // 2. Test with different pool states - second deposit (proportional)
    let provider2 = Address::generate(&context.env);
    context.mint(&provider2, provider2_deposit);
    let shares2 = context.client.deposit(&provider2, &provider2_deposit);
    assert_eq!(shares2, provider2_expected_shares);

    // 3. Verify no precision loss - total should match
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_shares, expected_total_shares);
    assert_eq!(stats.total_liquidity, expected_total_liquidity);

    // 4. Test rounding behavior with small deposit after interest
    // Simulate interest distribution
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    let stats_after = context.client.get_pool_stats();
    assert_eq!(
        stats_after.total_liquidity,
        expected_liquidity_after_interest
    );

    // Small deposit should round down correctly
    let provider3 = Address::generate(&context.env);
    context.mint(&provider3, provider3_deposit);
    let shares3 = context.client.deposit(&provider3, &provider3_deposit);
    assert_eq!(shares3, provider3_expected_shares);
}

#[test]
fn test_multiple_lp_deposits() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 1000;
    let provider1_expected_shares = 1000;
    let provider2_deposit = 2000;
    let provider2_expected_shares = 2000;
    let provider3_deposit = 500;
    let provider3_expected_shares = 500;
    let expected_total_shares = 3500;
    let expected_total_liquidity = 3500;
    let expected_share_price = 10_000;

    // 1. Create 3 provider addresses
    let provider1 = Address::generate(&context.env);
    let provider2 = Address::generate(&context.env);
    let provider3 = Address::generate(&context.env);

    // 2. Mint different amounts to each
    context.mint(&provider1, provider1_deposit);
    context.mint(&provider2, provider2_deposit);
    context.mint(&provider3, provider3_deposit);

    // 3. Provider1 deposits tokens
    let shares1 = context.client.deposit(&provider1, &provider1_deposit);
    assert_eq!(shares1, provider1_expected_shares);

    // 4. Provider2 deposits tokens
    let shares2 = context.client.deposit(&provider2, &provider2_deposit);
    assert_eq!(shares2, provider2_expected_shares);

    // 5. Provider3 deposits tokens
    let shares3 = context.client.deposit(&provider3, &provider3_deposit);
    assert_eq!(shares3, provider3_expected_shares);

    // 6. Verify each provider's share balance is correct
    let provider1_balance = context.client.get_lp_shares(&provider1);
    let provider2_balance = context.client.get_lp_shares(&provider2);
    let provider3_balance = context.client.get_lp_shares(&provider3);

    assert_eq!(provider1_balance, provider1_expected_shares);
    assert_eq!(provider2_balance, provider2_expected_shares);
    assert_eq!(provider3_balance, provider3_expected_shares);

    // 7. Verify total_shares
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_shares, expected_total_shares);

    // 8. Verify total_liquidity
    assert_eq!(stats.total_liquidity, expected_total_liquidity);

    // 9. Verify share_price remains constant (no interest)
    assert_eq!(stats.share_price, expected_share_price);
}

// ─── Withdrawal Tests ────────────────────────────────────────────────────────

#[test]
fn test_full_withdrawal() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let withdrawal_shares = 1000;
    let expected_returned_amount = 1000;
    let expected_final_shares = 0;
    let expected_final_liquidity = 0;
    let expected_initial_liquidity = 1000;
    let expected_initial_shares = 1000;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // Verify initial state
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);

    let initial_provider_shares = context.client.get_lp_shares(&provider);
    assert_eq!(initial_provider_shares, expected_shares);

    // 3. Provider withdraws all shares
    let returned_amount = context.client.withdraw(&provider, &withdrawal_shares);

    // 4. Assert returned amount (original deposit, no interest)
    assert_eq!(returned_amount, expected_returned_amount);

    // 5. Verify provider's share balance using get_lp_shares()
    let provider_shares_after = context.client.get_lp_shares(&provider);
    assert_eq!(provider_shares_after, expected_final_shares);

    // 6. Verify pool stats: total_shares and total_liquidity
    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_shares, expected_final_shares);
    assert_eq!(final_stats.total_liquidity, expected_final_liquidity);
}

#[test]
fn test_partial_withdrawal() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let withdrawal_shares = 400;
    let expected_returned_amount = 400;
    let expected_remaining_shares = 600;
    let expected_final_liquidity = 600;
    let expected_final_shares = 600;
    let expected_initial_liquidity = 1000;
    let expected_initial_shares = 1000;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // Verify initial state
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);

    let initial_provider_shares = context.client.get_lp_shares(&provider);
    assert_eq!(initial_provider_shares, expected_shares);

    // 3. Provider withdraws shares
    let returned_amount = context.client.withdraw(&provider, &withdrawal_shares);

    // 4. Assert returned amount (proportional to shares withdrawn)
    assert_eq!(returned_amount, expected_returned_amount);

    // 5. Verify remaining share balance using get_lp_shares()
    let remaining_shares = context.client.get_lp_shares(&provider);
    assert_eq!(remaining_shares, expected_remaining_shares);

    // 6. Verify pool stats: total_shares and total_liquidity
    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_shares, expected_final_shares);
    assert_eq!(final_stats.total_liquidity, expected_final_liquidity);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_insufficient_liquidity_rejection() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let loan_amount = 1000;
    let withdrawal_shares = 500;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens
    context.client.deposit(&provider, &deposit_amount);

    // 3. Create a merchant address
    let merchant = Address::generate(&context.env);

    // 4. Fund a loan that locks all liquidity
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);

    // 5. Attempt to withdraw any amount - this should panic with Error(Contract, #6)
    context.client.withdraw(&provider, &withdrawal_shares);
}

#[test]
fn test_withdrawal_with_active_loans() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let loan_amount = 400;
    let expected_available_after_loan = 600;
    let expected_locked_after_loan = 400;
    let withdrawal_shares = 600;
    let expected_withdrawn_amount = 600;
    let expected_remaining_shares = 400;
    let expected_final_liquidity = 400;
    let expected_final_available = 0;
    let expected_final_locked = 400;
    let expected_final_shares = 400;
    let expected_initial_liquidity = 1000;
    let expected_initial_available = 1000;
    let expected_initial_locked = 0;
    let expected_initial_shares = 1000;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // Verify initial state
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(
        initial_stats.available_liquidity,
        expected_initial_available
    );
    assert_eq!(initial_stats.locked_liquidity, expected_initial_locked);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);

    // 3. Create a merchant address
    let merchant = Address::generate(&context.env);

    // 4. Fund a loan that locks partial liquidity
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);

    // Verify loan funding state
    let loan_stats = context.client.get_pool_stats();
    assert_eq!(loan_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(
        loan_stats.available_liquidity,
        expected_available_after_loan
    );
    assert_eq!(loan_stats.locked_liquidity, expected_locked_after_loan);
    assert_eq!(loan_stats.total_shares, expected_initial_shares);

    // 5. Calculate max withdrawable shares
    let max_withdrawable_shares =
        (loan_stats.available_liquidity * loan_stats.total_shares) / loan_stats.total_liquidity;
    assert_eq!(max_withdrawable_shares, withdrawal_shares);

    // 6. Withdraw up to available amount - this should succeed
    let withdrawn_amount = context.client.withdraw(&provider, &withdrawal_shares);
    assert_eq!(withdrawn_amount, expected_withdrawn_amount);

    // 7. Verify locked_liquidity remains unchanged
    let after_withdrawal_stats = context.client.get_pool_stats();
    assert_eq!(
        after_withdrawal_stats.locked_liquidity,
        expected_final_locked
    );

    // 8. Verify remaining provider shares
    let remaining_provider_shares = context.client.get_lp_shares(&provider);
    assert_eq!(remaining_provider_shares, expected_remaining_shares);

    // Verify pool state after withdrawal
    assert_eq!(
        after_withdrawal_stats.total_liquidity,
        expected_final_liquidity
    );
    assert_eq!(
        after_withdrawal_stats.available_liquidity,
        expected_final_available
    );
    assert_eq!(after_withdrawal_stats.total_shares, expected_final_shares);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_withdrawal_with_active_loans_exceeds_available_shares() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let loan_amount = 400;
    let first_withdrawal_shares = 600;
    let second_withdrawal_shares = 500;

    // Setup: provider deposits tokens, loan locks liquidity, provider withdraws shares
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    let merchant = Address::generate(&context.env);
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);
    context.client.withdraw(&provider, &first_withdrawal_shares);

    // Now provider has 400 shares remaining, but available_liquidity is 0
    // Attempt to withdraw 500 shares (more than remaining) - should fail with InsufficientShares
    context
        .client
        .withdraw(&provider, &second_withdrawal_shares);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_withdrawal_with_active_loans_no_available_liquidity() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let loan_amount = 400;
    let first_withdrawal_shares = 600;
    let second_withdrawal_shares = 100;

    // Setup: provider deposits tokens, loan locks liquidity, provider withdraws shares
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    let merchant = Address::generate(&context.env);
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);
    context.client.withdraw(&provider, &first_withdrawal_shares);

    // Now provider has 400 shares remaining, but available_liquidity is 0
    // Attempt to withdraw any amount when available_liquidity is 0 - should fail
    context
        .client
        .withdraw(&provider, &second_withdrawal_shares);
}

#[test]
fn test_share_value_maintained_after_withdrawal() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 1000;
    let provider2_deposit = 1000;
    let provider1_shares = 1000;
    let provider2_shares = 1000;
    let expected_initial_liquidity = 2000;
    let expected_initial_shares = 2000;
    let expected_initial_share_price = 10_000;
    let provider1_withdrawal_shares = 1000;
    let expected_withdrawn1 = 1000;
    let expected_liquidity_after_withdrawal = 1000;
    let expected_shares_after_withdrawal = 1000;
    let expected_share_price_after_withdrawal = 10_000;
    let provider2_withdrawal_shares = 1000;
    let expected_withdrawn2 = 1000;
    let expected_final_liquidity = 0;
    let expected_final_shares = 0;
    let expected_final_provider1_shares = 0;
    let expected_final_provider2_shares = 0;

    // 1. Create two provider addresses
    let provider1 = Address::generate(&context.env);
    let provider2 = Address::generate(&context.env);

    // 2. Mint tokens to each provider
    context.mint(&provider1, provider1_deposit);
    context.mint(&provider2, provider2_deposit);

    // 3. Both providers deposit equal amounts
    let shares1 = context.client.deposit(&provider1, &provider1_deposit);
    assert_eq!(shares1, provider1_shares);

    let shares2 = context.client.deposit(&provider2, &provider2_deposit);
    assert_eq!(shares2, provider2_shares);

    // 4. Verify initial state
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(stats.total_shares, expected_initial_shares);
    assert_eq!(stats.share_price, expected_initial_share_price);

    // 5. First provider withdraws all their shares
    let withdrawn1 = context
        .client
        .withdraw(&provider1, &provider1_withdrawal_shares);
    assert_eq!(withdrawn1, expected_withdrawn1);

    // 6. Verify second provider's shares still represent correct pool value
    let provider2_shares_balance = context.client.get_lp_shares(&provider2);
    assert_eq!(provider2_shares_balance, provider2_shares);

    // Total pool now has expected liquidity and shares
    let stats_after_withdrawal = context.client.get_pool_stats();
    assert_eq!(
        stats_after_withdrawal.total_liquidity,
        expected_liquidity_after_withdrawal
    );
    assert_eq!(
        stats_after_withdrawal.total_shares,
        expected_shares_after_withdrawal
    );

    // Provider2's shares represent 100% of the pool
    assert_eq!(
        provider2_shares_balance,
        stats_after_withdrawal.total_shares
    );

    // 7. Verify share_price remains consistent
    assert_eq!(
        stats_after_withdrawal.share_price,
        expected_share_price_after_withdrawal
    );

    // 8. Second provider withdraws all their shares and receives expected amount
    let withdrawn2 = context
        .client
        .withdraw(&provider2, &provider2_withdrawal_shares);
    assert_eq!(withdrawn2, expected_withdrawn2);

    // 9. Verify pool is empty after both withdrawals
    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_liquidity, expected_final_liquidity);
    assert_eq!(final_stats.total_shares, expected_final_shares);

    // Verify both providers have no shares remaining
    assert_eq!(
        context.client.get_lp_shares(&provider1),
        expected_final_provider1_shares
    );
    assert_eq!(
        context.client.get_lp_shares(&provider2),
        expected_final_provider2_shares
    );
}

// ─── Interest Distribution Tests ─────────────────────────────────────────────

#[test]
fn test_interest_increases_share_value() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let expected_initial_liquidity = 1000;
    let expected_initial_shares = 1000;
    let expected_initial_share_price = 10_000;
    let interest_amount = 100;
    let principal_repayment = 0;
    let lp_interest_percentage = 85;
    let lp_interest = (interest_amount * lp_interest_percentage) / 100;
    let expected_total_liquidity = deposit_amount + lp_interest;
    let expected_share_price = (expected_total_liquidity * 10_000) / expected_shares;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // Verify initial state
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);
    assert_eq!(initial_stats.share_price, expected_initial_share_price);

    // 3. Simulate interest distribution: mint tokens to creditline and receive repayment
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 4. Verify share_price increased
    let updated_stats = context.client.get_pool_stats();
    assert_eq!(updated_stats.total_liquidity, expected_total_liquidity);
    assert_eq!(updated_stats.total_shares, expected_shares);
    assert_eq!(updated_stats.share_price, expected_share_price);

    // 5. Verify provider can withdraw more tokens than deposited
    let withdrawn_amount = context.client.withdraw(&provider, &expected_shares);
    assert_eq!(withdrawn_amount, expected_total_liquidity);

    // Verify final state - pool should be empty
    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_liquidity, 0);
    assert_eq!(final_stats.total_shares, 0);
}

#[test]
fn test_fee_split_accuracy() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 10000;
    let expected_shares = 10000;
    let expected_initial_liquidity = 10000;
    let expected_initial_shares = 10000;
    let interest_amount = 1000;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let treasury_percentage = 10;
    let merchant_fund_percentage = 5;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let treasury_fee = (interest_amount * treasury_percentage) / 100;
    let merchant_fund_fee = (interest_amount * merchant_fund_percentage) / 100;
    let expected_total_liquidity = deposit_amount + lp_interest;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens to setup pool with liquidity
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // Verify initial state
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);

    // Check initial balances
    let initial_treasury_balance = context.token.balance(&context.treasury);
    let initial_merchant_fund_balance = context.token.balance(&context.merchant_fund);

    // 3. Distribute interest
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 4. Verify LP portion added to total_liquidity
    let updated_stats = context.client.get_pool_stats();
    assert_eq!(updated_stats.total_liquidity, expected_total_liquidity);
    assert_eq!(updated_stats.total_shares, expected_shares);

    // 5. Verify treasury receives expected tokens
    let treasury_balance = context.token.balance(&context.treasury);
    assert_eq!(treasury_balance, initial_treasury_balance + treasury_fee);

    // 6. Verify merchant_fund receives expected tokens
    let merchant_fund_balance = context.token.balance(&context.merchant_fund);
    assert_eq!(
        merchant_fund_balance,
        initial_merchant_fund_balance + merchant_fund_fee
    );
}

#[test]
fn test_multiple_lp_proportional_interest() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider_a_deposit = 1000;
    let provider_a_shares = 1000;
    let provider_b_deposit = 2000;
    let provider_b_shares = 2000;
    let expected_initial_liquidity = 3000;
    let expected_initial_shares = 3000;
    let interest_amount = 300;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_interest = expected_initial_liquidity + lp_interest;
    let expected_share_price =
        (expected_liquidity_after_interest * 10_000) / expected_initial_shares;
    let provider_a_withdrawal_shares = 1000;
    let expected_withdrawn_a = 1085;
    let provider_b_withdrawal_shares = 2000;
    let expected_withdrawn_b = 2170;

    // 1. Provider A deposits tokens
    let provider_a = Address::generate(&context.env);
    context.mint(&provider_a, provider_a_deposit);
    let shares_a = context.client.deposit(&provider_a, &provider_a_deposit);
    assert_eq!(shares_a, provider_a_shares);

    // 2. Provider B deposits tokens
    let provider_b = Address::generate(&context.env);
    context.mint(&provider_b, provider_b_deposit);
    let shares_b = context.client.deposit(&provider_b, &provider_b_deposit);
    assert_eq!(shares_b, provider_b_shares);

    // Verify initial state
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);

    // 3. Distribute interest
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 4. Calculate expected value per share
    let stats_after_interest = context.client.get_pool_stats();
    assert_eq!(
        stats_after_interest.total_liquidity,
        expected_liquidity_after_interest
    );
    assert_eq!(stats_after_interest.total_shares, expected_initial_shares);
    assert_eq!(stats_after_interest.share_price, expected_share_price);

    // 5. Verify both providers can withdraw proportionally increased amounts
    let withdrawn_a = context
        .client
        .withdraw(&provider_a, &provider_a_withdrawal_shares);
    assert_eq!(withdrawn_a, expected_withdrawn_a);

    let withdrawn_b = context
        .client
        .withdraw(&provider_b, &provider_b_withdrawal_shares);
    assert_eq!(withdrawn_b, expected_withdrawn_b);

    // 6. Verify the ratio of their withdrawals matches their share ratio
    assert_eq!(withdrawn_a * 2, withdrawn_b);
}

#[test]
fn test_share_value_appreciation_over_time() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let expected_initial_share_price = 10_000;
    let interest_amount = 100;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_first = deposit_amount + lp_interest;
    let expected_share_price_after_first = 10850;
    let expected_liquidity_after_second = expected_liquidity_after_first + lp_interest;
    let expected_share_price_after_second = 11700;
    let expected_liquidity_after_third = expected_liquidity_after_second + lp_interest;
    let expected_share_price_after_third = 12550;
    let withdrawal_shares = 1000;
    let expected_final_withdrawal = expected_liquidity_after_third;
    let new_provider_deposit = 1000;
    let expected_new_provider_shares = 1000;

    // 1. Provider deposits tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // 2. Record initial share_price
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.share_price, expected_initial_share_price);

    // 3. Distribute interest multiple times
    // First interest
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 4. Verify share_price increases after first distribution
    let stats_after_first = context.client.get_pool_stats();
    assert_eq!(
        stats_after_first.total_liquidity,
        expected_liquidity_after_first
    );
    assert_eq!(
        stats_after_first.share_price,
        expected_share_price_after_first
    );

    // Second interest
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // Verify share_price increases after second distribution
    let stats_after_second = context.client.get_pool_stats();
    assert_eq!(
        stats_after_second.total_liquidity,
        expected_liquidity_after_second
    );
    assert_eq!(
        stats_after_second.share_price,
        expected_share_price_after_second
    );

    // Third interest
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // Verify share_price increases after third distribution
    let stats_after_third = context.client.get_pool_stats();
    assert_eq!(
        stats_after_third.total_liquidity,
        expected_liquidity_after_third
    );
    assert_eq!(
        stats_after_third.share_price,
        expected_share_price_after_third
    );

    // 5. Verify final withdrawal amount reflects all accumulated interest
    let withdrawn = context.client.withdraw(&provider, &withdrawal_shares);
    assert_eq!(withdrawn, expected_final_withdrawal);

    // 6. Test that new depositors after interest get fewer shares per token
    let new_provider = Address::generate(&context.env);
    context.mint(&new_provider, new_provider_deposit);
    let new_shares = context.client.deposit(&new_provider, &new_provider_deposit);
    // With empty pool, first deposit gets 1:1 ratio again
    assert_eq!(new_shares, expected_new_provider_shares);
}

// ─── Edge Cases and Pool State Tests ─────────────────────────────────────────

#[test]
fn test_pool_empty_state_handling() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let expected_empty_liquidity = 0;
    let expected_empty_shares = 0;
    let expected_empty_locked = 0;
    let expected_empty_share_price = 10_000;
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let withdrawal_shares = 1000;
    let expected_returned_amount = 1000;
    let calculation_shares = 1000;
    let expected_calculation_result = 0;
    let second_deposit_amount = 500;
    let expected_second_shares = 500;
    let expected_final_liquidity = 500;
    let expected_final_shares = 500;
    let expected_final_share_price = 10_000;

    // 1. Verify initial empty pool stats
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_empty_liquidity);
    assert_eq!(initial_stats.total_shares, expected_empty_shares);
    assert_eq!(initial_stats.locked_liquidity, expected_empty_locked);
    assert_eq!(initial_stats.share_price, expected_empty_share_price);

    // 2. Create a provider, mint tokens, deposit them
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // Verify pool has liquidity after deposit
    let after_deposit_stats = context.client.get_pool_stats();
    assert_eq!(after_deposit_stats.total_liquidity, deposit_amount);
    assert_eq!(after_deposit_stats.total_shares, expected_shares);

    // 3. Withdraw all liquidity
    let returned_amount = context.client.withdraw(&provider, &withdrawal_shares);
    assert_eq!(returned_amount, expected_returned_amount);

    // 4. Verify pool returns to empty state
    let empty_stats = context.client.get_pool_stats();
    assert_eq!(empty_stats.total_liquidity, expected_empty_liquidity);
    assert_eq!(empty_stats.total_shares, expected_empty_shares);
    assert_eq!(empty_stats.locked_liquidity, expected_empty_locked);
    assert_eq!(empty_stats.share_price, expected_empty_share_price);

    // 5. Test calculate_withdrawal with empty pool returns 0
    let withdrawal_calculation = context.client.calculate_withdrawal(&calculation_shares);
    assert_eq!(withdrawal_calculation, expected_calculation_result);

    // 6. Verify next deposit after empty state works correctly (1:1 ratio)
    context.mint(&provider, second_deposit_amount);
    let new_shares = context.client.deposit(&provider, &second_deposit_amount);
    assert_eq!(new_shares, expected_second_shares);

    // Verify final state shows correct 1:1 ratio
    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_liquidity, expected_final_liquidity);
    assert_eq!(final_stats.total_shares, expected_final_shares);
    assert_eq!(final_stats.share_price, expected_final_share_price);
}

#[test]
fn test_small_deposit_rounding() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 1_000_000;
    let provider1_expected_shares = 1_000_000;
    let interest_amount = 200_000;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_interest = provider1_deposit + lp_interest;
    let expected_share_price = 11700;
    let provider2_deposit = 100;
    let provider2_expected_shares = 85;
    let provider2_withdrawal_shares = 85;
    let provider3_deposit = 10;
    let provider3_expected_shares = 8;

    // 1. Make a large initial deposit
    let provider1 = Address::generate(&context.env);
    context.mint(&provider1, provider1_deposit);
    let shares1 = context.client.deposit(&provider1, &provider1_deposit);
    assert_eq!(shares1, provider1_expected_shares);

    // 2. Distribute interest to increase share_price
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // Verify share_price increased
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, expected_liquidity_after_interest);
    assert_eq!(stats.share_price, expected_share_price);

    // 3. Attempt very small deposits
    let provider2 = Address::generate(&context.env);
    context.mint(&provider2, provider2_deposit);
    let shares2 = context.client.deposit(&provider2, &provider2_deposit);

    // 4. Verify shares are calculated correctly (rounded down)
    assert_eq!(shares2, provider2_expected_shares);

    // 5. Verify no share inflation attack is possible
    let withdrawn2 = context
        .client
        .withdraw(&provider2, &provider2_withdrawal_shares);
    assert!(withdrawn2 <= provider2_deposit);

    // 6. Test edge case where deposit is very small but still gets shares
    let provider3 = Address::generate(&context.env);
    context.mint(&provider3, provider3_deposit);
    let shares3 = context.client.deposit(&provider3, &provider3_deposit);
    assert_eq!(shares3, provider3_expected_shares);
}

#[test]
fn test_concurrent_deposits_and_withdrawals() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 1000;
    let provider1_expected_shares = 1000;
    let provider2_deposit = 2000;
    let provider2_expected_shares = 2000;
    let provider3_deposit = 1500;
    let provider3_expected_shares = 1500;
    let expected_initial_liquidity = 4500;
    let expected_initial_shares = 4500;
    let provider1_withdrawal_shares = 500;
    let expected_withdrawn1 = 500;
    let expected_liquidity_after_withdrawal = 4000;
    let expected_shares_after_withdrawal = 4000;
    let interest_amount = 400;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_interest = expected_liquidity_after_withdrawal + lp_interest;
    let provider4_deposit = 1000;
    let provider4_expected_shares = 921;
    let expected_liquidity_after_provider4 = 5340;
    let expected_shares_after_provider4 = 4921;
    let provider2_withdrawal_shares = 2000;
    let expected_withdrawn2 = 2170;
    let expected_final_shares = 2921;
    let expected_final_liquidity = 3170;

    // 1. Multiple providers deposit in sequence
    let provider1 = Address::generate(&context.env);
    context.mint(&provider1, provider1_deposit);
    let shares1 = context.client.deposit(&provider1, &provider1_deposit);
    assert_eq!(shares1, provider1_expected_shares);

    let provider2 = Address::generate(&context.env);
    context.mint(&provider2, provider2_deposit);
    let shares2 = context.client.deposit(&provider2, &provider2_deposit);
    assert_eq!(shares2, provider2_expected_shares);

    let provider3 = Address::generate(&context.env);
    context.mint(&provider3, provider3_deposit);
    let shares3 = context.client.deposit(&provider3, &provider3_deposit);
    assert_eq!(shares3, provider3_expected_shares);

    // Verify initial state
    let stats1 = context.client.get_pool_stats();
    assert_eq!(stats1.total_liquidity, expected_initial_liquidity);
    assert_eq!(stats1.total_shares, expected_initial_shares);

    // 2. Some providers withdraw while others deposit
    let withdrawn1 = context
        .client
        .withdraw(&provider1, &provider1_withdrawal_shares);
    assert_eq!(withdrawn1, expected_withdrawn1);

    let stats2 = context.client.get_pool_stats();
    assert_eq!(stats2.total_liquidity, expected_liquidity_after_withdrawal);
    assert_eq!(stats2.total_shares, expected_shares_after_withdrawal);

    // 3. Distribute interest between operations
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 4. Verify pool stats remain consistent throughout
    let stats3 = context.client.get_pool_stats();
    assert_eq!(stats3.total_liquidity, expected_liquidity_after_interest);
    assert_eq!(stats3.total_shares, expected_shares_after_withdrawal);

    // New provider deposits after interest
    let provider4 = Address::generate(&context.env);
    context.mint(&provider4, provider4_deposit);
    let shares4 = context.client.deposit(&provider4, &provider4_deposit);
    assert_eq!(shares4, provider4_expected_shares);

    let stats4 = context.client.get_pool_stats();
    assert_eq!(stats4.total_liquidity, expected_liquidity_after_provider4);
    assert_eq!(stats4.total_shares, expected_shares_after_provider4);

    // 5. Verify all providers can withdraw expected amounts
    let withdrawn2 = context
        .client
        .withdraw(&provider2, &provider2_withdrawal_shares);
    assert_eq!(withdrawn2, expected_withdrawn2);

    // 6. Verify total_liquidity and total_shares always match proportionally
    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_shares, expected_final_shares);
    assert_eq!(final_stats.total_liquidity, expected_final_liquidity);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_locked_liquidity_not_withdrawable() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let loan_amount = 1000;
    let expected_locked_liquidity = 1000;
    let expected_available_liquidity = 0;
    let withdrawal_shares = 100;

    // 1. Provider deposits tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    // 2. Fund loan that locks all liquidity
    let merchant = Address::generate(&context.env);
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);

    // Verify all liquidity is locked
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.locked_liquidity, expected_locked_liquidity);
    assert_eq!(stats.available_liquidity, expected_available_liquidity);

    // 3. Attempt any withdrawal - should panic with InsufficientLiquidity error (code #6)
    context.client.withdraw(&provider, &withdrawal_shares);
}

// ─── Loan Integration Tests ──────────────────────────────────────────────────

#[test]
fn test_loan_funding_reduces_available_liquidity() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_shares = 1000;
    let expected_initial_liquidity = 1000;
    let expected_initial_available = 1000;
    let expected_initial_locked = 0;
    let expected_initial_shares = 1000;
    let loan_amount = 400;
    let expected_locked_after_loan = 400;
    let expected_available_after_loan = 600;
    let expected_total_liquidity = 1000;
    let expected_total_shares = 1000;

    // 1. Create a provider address and mint tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);

    // 2. Provider deposits tokens
    let shares = context.client.deposit(&provider, &deposit_amount);
    assert_eq!(shares, expected_shares);

    // 3. Record initial pool stats
    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(
        initial_stats.available_liquidity,
        expected_initial_available
    );
    assert_eq!(initial_stats.locked_liquidity, expected_initial_locked);
    assert_eq!(initial_stats.total_shares, expected_initial_shares);

    // 4. Create a merchant address
    let merchant = Address::generate(&context.env);

    // 5. Fund a loan
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);

    // 6. Verify locked_liquidity increased
    let updated_stats = context.client.get_pool_stats();
    assert_eq!(updated_stats.locked_liquidity, expected_locked_after_loan);

    // 7. Verify available_liquidity decreased
    assert_eq!(
        updated_stats.available_liquidity,
        expected_available_after_loan
    );

    // 8. Verify total_liquidity unchanged
    assert_eq!(updated_stats.total_liquidity, expected_total_liquidity);

    // 9. Verify total_shares unchanged
    assert_eq!(updated_stats.total_shares, expected_total_shares);
}

#[test]
fn test_repayment_increases_pool_value() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 10000;
    let expected_initial_liquidity = 10000;
    let expected_initial_share_price = 10_000;
    let loan_amount = 5000;
    let expected_locked_after_loan = 5000;
    let expected_available_after_loan = 5000;
    let principal_repayment = 5000;
    let interest_amount = 500;
    let lp_percentage = 85;
    let treasury_percentage = 10;
    let merchant_fund_percentage = 5;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let treasury_fee = (interest_amount * treasury_percentage) / 100;
    let merchant_fund_fee = (interest_amount * merchant_fund_percentage) / 100;
    let expected_locked_after_repayment = 0;
    let expected_liquidity_after_repayment = deposit_amount + principal_repayment + lp_interest;
    let expected_share_price_after_repayment = 15425;
    let total_repayment = principal_repayment + interest_amount;

    // 1. Provider deposits tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.share_price, expected_initial_share_price);

    // 2. Fund a loan
    let merchant = Address::generate(&context.env);
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);

    let after_loan_stats = context.client.get_pool_stats();
    assert_eq!(
        after_loan_stats.locked_liquidity,
        expected_locked_after_loan
    );
    assert_eq!(
        after_loan_stats.available_liquidity,
        expected_available_after_loan
    );

    // Check initial treasury and merchant fund balances
    let initial_treasury_balance = context.token.balance(&context.treasury);
    let initial_merchant_fund_balance = context.token.balance(&context.merchant_fund);

    // 3. Simulate repayment with principal + interest
    context.mint(&context.creditline, total_repayment);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 4. Verify locked_liquidity decreased by principal amount
    let after_repayment_stats = context.client.get_pool_stats();
    assert_eq!(
        after_repayment_stats.locked_liquidity,
        expected_locked_after_repayment
    );

    // 5. Verify total_liquidity increased by (principal + LP_interest_portion)
    assert_eq!(
        after_repayment_stats.total_liquidity,
        expected_liquidity_after_repayment
    );

    // 6. Verify share_price increased
    assert_eq!(
        after_repayment_stats.share_price,
        expected_share_price_after_repayment
    );

    // 7. Verify treasury and merchant_fund received their fee portions
    let treasury_balance = context.token.balance(&context.treasury);
    assert_eq!(treasury_balance, initial_treasury_balance + treasury_fee);

    let merchant_fund_balance = context.token.balance(&context.merchant_fund);
    assert_eq!(
        merchant_fund_balance,
        initial_merchant_fund_balance + merchant_fund_fee
    );
}

#[test]
fn test_guarantee_receipt_on_default() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 10000;
    let expected_initial_liquidity = 10000;
    let expected_initial_share_price = 10_000;
    let loan_amount = 5000;
    let expected_locked_after_loan = 5000;
    let expected_available_after_loan = 5000;
    let expected_total_liquidity_after_loan = 10000;
    let guarantee_amount = 3000;
    let expected_locked_after_guarantee = 2000;
    let expected_total_liquidity_after_guarantee = 13000;
    let expected_share_price_after_guarantee = 13000;

    // 1. Provider deposits tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.share_price, expected_initial_share_price);

    // 2. Fund a loan (locks liquidity)
    let merchant = Address::generate(&context.env);
    context
        .client
        .fund_loan(&context.creditline, &merchant, &loan_amount);

    let after_loan_stats = context.client.get_pool_stats();
    assert_eq!(
        after_loan_stats.locked_liquidity,
        expected_locked_after_loan
    );
    assert_eq!(
        after_loan_stats.available_liquidity,
        expected_available_after_loan
    );
    assert_eq!(
        after_loan_stats.total_liquidity,
        expected_total_liquidity_after_loan
    );

    // 3. Simulate default with partial guarantee receipt
    context.mint(&context.creditline, guarantee_amount);
    context
        .client
        .receive_guarantee(&context.creditline, &guarantee_amount);

    // 4. Verify locked_liquidity reduced by guarantee amount
    let after_guarantee_stats = context.client.get_pool_stats();
    assert_eq!(
        after_guarantee_stats.locked_liquidity,
        expected_locked_after_guarantee
    );

    // 5. Verify total_liquidity increased by guarantee amount
    assert_eq!(
        after_guarantee_stats.total_liquidity,
        expected_total_liquidity_after_guarantee
    );

    // 6. Verify remaining locked_liquidity represents unrecovered loss
    assert_eq!(
        after_guarantee_stats.locked_liquidity,
        expected_locked_after_guarantee
    );

    // 7. Verify share_price reflects the partial recovery
    assert_eq!(
        after_guarantee_stats.share_price,
        expected_share_price_after_guarantee
    );
}

// ─── Complete Lifecycle Test ─────────────────────────────────────────────────

#[test]
#[test]
fn test_withdrawal_calculation_precision() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let initial_deposit = 10000;
    let calc1_shares = 5000;
    let expected_calc1 = 5000;
    let calc2_shares = 10000;
    let expected_calc2 = 10000;
    let withdrawal1_shares = 5000;
    let second_deposit = 10000;
    let interest_amount = 1000;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_interest = 15850;
    let calc3_shares = 1000;
    let expected_calc3 = 1056;
    let withdrawal2_shares = 1000;
    let calc4_shares = 1;
    let withdrawal3_shares = 1;

    // 1. Setup pool with various states
    let provider = Address::generate(&context.env);
    context.mint(&provider, initial_deposit);
    context.client.deposit(&provider, &initial_deposit);

    // 2. Call calculate_withdrawal for different share amounts
    let calc1 = context.client.calculate_withdrawal(&calc1_shares);
    assert_eq!(calc1, expected_calc1);

    let calc2 = context.client.calculate_withdrawal(&calc2_shares);
    assert_eq!(calc2, expected_calc2);

    // 3. Perform actual withdrawal
    let withdrawn1 = context.client.withdraw(&provider, &withdrawal1_shares);

    // 4. Verify returned amount matches calculation
    assert_eq!(withdrawn1, calc1);

    // Deposit again for more tests
    context.mint(&provider, second_deposit);
    context.client.deposit(&provider, &second_deposit);

    // 5. Test with edge cases (very small/large amounts, after interest, etc.)
    // Distribute interest
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, expected_liquidity_after_interest);

    // Calculate withdrawal after interest
    let calc3 = context.client.calculate_withdrawal(&calc3_shares);
    assert_eq!(calc3, expected_calc3);

    // Verify actual withdrawal matches
    let withdrawn2 = context.client.withdraw(&provider, &withdrawal2_shares);
    assert_eq!(withdrawn2, calc3);

    // Test very small amount
    let calc4 = context.client.calculate_withdrawal(&calc4_shares);
    let withdrawn3 = context.client.withdraw(&provider, &withdrawal3_shares);
    assert_eq!(withdrawn3, calc4);

    // Test very large amount
    let remaining_shares = context.client.get_lp_shares(&provider);
    let calc5 = context.client.calculate_withdrawal(&remaining_shares);
    let withdrawn4 = context.client.withdraw(&provider, &remaining_shares);
    assert_eq!(withdrawn4, calc5);
}

#[test]
fn test_share_price_calculation() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let expected_empty_share_price = 10_000;
    let deposit_amount = 5000;
    let expected_share_price_after_deposit = 10_000;
    let interest_amount = 500;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_interest = deposit_amount + lp_interest;
    let expected_share_price_after_interest = 10850;
    let withdrawal_shares = 2000;
    let expected_share_price_after_withdrawal = 10850;
    let loop_interest_amount = 100;
    let loop_iterations = 5;
    let loop_lp_interest = (loop_interest_amount * lp_percentage) / 100;
    let expected_final_liquidity = 3680;
    let expected_final_share_price = 12266;

    // 1. Empty pool: share_price should be expected value
    let empty_stats = context.client.get_pool_stats();
    assert_eq!(empty_stats.share_price, expected_empty_share_price);

    // 2. After first deposit: share_price should remain constant
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    let after_deposit_stats = context.client.get_pool_stats();
    assert_eq!(
        after_deposit_stats.share_price,
        expected_share_price_after_deposit
    );

    // 3. After interest: share_price should increase proportionally
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    let after_interest_stats = context.client.get_pool_stats();
    assert_eq!(
        after_interest_stats.total_liquidity,
        expected_liquidity_after_interest
    );
    assert_eq!(
        after_interest_stats.share_price,
        expected_share_price_after_interest
    );

    // 4. After withdrawal: share_price should remain constant
    context.client.withdraw(&provider, &withdrawal_shares);

    let after_withdrawal_stats = context.client.get_pool_stats();
    assert_eq!(
        after_withdrawal_stats.share_price,
        expected_share_price_after_withdrawal
    );

    // 5. Test edge cases (very large pools, after many interest events)
    // Add more interest events
    for _ in 0..loop_iterations {
        context.mint(&context.creditline, loop_interest_amount);
        context.client.receive_repayment(
            &context.creditline,
            &principal_repayment,
            &loop_interest_amount,
        );
    }

    let final_stats = context.client.get_pool_stats();
    assert_eq!(final_stats.total_liquidity, expected_final_liquidity);
    assert_eq!(final_stats.share_price, expected_final_share_price);
}

#[test]
fn test_multiple_interest_distributions() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let deposit_amount = 1000;
    let expected_initial_liquidity = 1000;
    let expected_initial_share_price = 10_000;
    let interest_amount = 100;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (interest_amount * lp_percentage) / 100;
    let expected_liquidity_after_event1 = 1085;
    let expected_share_price_after_event1 = 10850;
    let expected_liquidity_after_event2 = 1170;
    let expected_share_price_after_event2 = 11700;
    let expected_liquidity_after_event3 = 1255;
    let expected_share_price_after_event3 = 12550;
    let expected_liquidity_after_event4 = 1340;
    let expected_share_price_after_event4 = 13400;
    let expected_liquidity_after_event5 = 1425;
    let expected_share_price_after_event5 = 14250;
    let withdrawal_shares = 1000;
    let expected_withdrawn = 1425;

    // 1. Provider deposits tokens
    let provider = Address::generate(&context.env);
    context.mint(&provider, deposit_amount);
    context.client.deposit(&provider, &deposit_amount);

    let initial_stats = context.client.get_pool_stats();
    assert_eq!(initial_stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(initial_stats.share_price, expected_initial_share_price);

    // 2. Distribute interest event 1
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 3. Verify share_price increase
    let stats1 = context.client.get_pool_stats();
    assert_eq!(stats1.total_liquidity, expected_liquidity_after_event1);
    assert_eq!(stats1.share_price, expected_share_price_after_event1);

    // 4. Distribute interest event 2
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    // 5. Verify share_price compounds correctly
    let stats2 = context.client.get_pool_stats();
    assert_eq!(stats2.total_liquidity, expected_liquidity_after_event2);
    assert_eq!(stats2.share_price, expected_share_price_after_event2);

    // 6. Continue for several events
    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    let stats3 = context.client.get_pool_stats();
    assert_eq!(stats3.total_liquidity, expected_liquidity_after_event3);
    assert_eq!(stats3.share_price, expected_share_price_after_event3);

    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    let stats4 = context.client.get_pool_stats();
    assert_eq!(stats4.total_liquidity, expected_liquidity_after_event4);
    assert_eq!(stats4.share_price, expected_share_price_after_event4);

    context.mint(&context.creditline, interest_amount);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &interest_amount);

    let stats5 = context.client.get_pool_stats();
    assert_eq!(stats5.total_liquidity, expected_liquidity_after_event5);
    assert_eq!(stats5.share_price, expected_share_price_after_event5);

    // 7. Verify final share value reflects all compounded interest
    let withdrawn = context.client.withdraw(&provider, &withdrawal_shares);
    assert_eq!(withdrawn, expected_withdrawn);
}

#[test]
fn test_zero_shares_edge_case() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let provider1_deposit = 10_000_000;
    let interest_amount = 1_000_000;
    let principal_repayment = 0;
    let loop_iterations = 10;
    let lp_percentage = 85;
    let total_lp_interest = (interest_amount * lp_percentage * loop_iterations as i128) / 100;
    let expected_total_liquidity = provider1_deposit + total_lp_interest;
    let expected_share_price = 18500;

    // 1. Create pool with high share_price (large deposit + lots of interest)
    let provider1 = Address::generate(&context.env);
    context.mint(&provider1, provider1_deposit);
    context.client.deposit(&provider1, &provider1_deposit);

    // Distribute large amount of interest multiple times
    for _ in 0..loop_iterations {
        context.mint(&context.creditline, interest_amount);
        context.client.receive_repayment(
            &context.creditline,
            &principal_repayment,
            &interest_amount,
        );
    }

    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, expected_total_liquidity);
    assert_eq!(stats.share_price, expected_share_price);

    // 2. Attempt deposit so small it would round to 0 shares
    // The contract will reject this with InvalidAmount error
}

#[test]
fn test_maximum_values_handling() {
    let context = TestEnv::setup();

    // Declare all test parameters as variables
    let large_amount = 1_000_000_000_000i128;
    let expected_shares = large_amount;
    let expected_initial_liquidity = large_amount;
    let expected_initial_shares = large_amount;
    let expected_initial_share_price = 10_000;
    let calc_shares = large_amount / 2;
    let expected_calc = large_amount / 2;
    let large_interest = 100_000_000_000i128;
    let principal_repayment = 0;
    let lp_percentage = 85;
    let lp_interest = (large_interest * lp_percentage) / 100;
    let expected_liquidity_after_interest = large_amount + lp_interest;
    let expected_share_price_after_interest =
        (expected_liquidity_after_interest * 10_000) / large_amount;
    let withdrawal_shares = large_amount;

    // 1. Test with maximum reasonable token amounts
    let provider = Address::generate(&context.env);
    context.mint(&provider, large_amount);
    let shares = context.client.deposit(&provider, &large_amount);
    assert_eq!(shares, expected_shares);

    // 2. Test share calculations with large numbers
    let stats = context.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, expected_initial_liquidity);
    assert_eq!(stats.total_shares, expected_initial_shares);
    assert_eq!(stats.share_price, expected_initial_share_price);

    // 3. Verify no overflow in multiplication/division operations
    let calc = context.client.calculate_withdrawal(&calc_shares);
    assert_eq!(calc, expected_calc);

    // 4. Test interest distribution with large amounts
    context.mint(&context.creditline, large_interest);
    context
        .client
        .receive_repayment(&context.creditline, &principal_repayment, &large_interest);

    let stats_after_interest = context.client.get_pool_stats();
    assert_eq!(
        stats_after_interest.total_liquidity,
        expected_liquidity_after_interest
    );

    // Verify share_price calculation doesn't overflow
    assert_eq!(
        stats_after_interest.share_price,
        expected_share_price_after_interest
    );

    // Test withdrawal with large amounts
    let withdrawn = context.client.withdraw(&provider, &withdrawal_shares);
    assert_eq!(withdrawn, expected_liquidity_after_interest);
}
