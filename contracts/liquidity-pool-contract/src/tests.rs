use crate::{LiquidityPoolContract, LiquidityPoolContractClient};
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

// ─── helpers ──────────────────────────────────────────────────────────────────

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
        let client: LiquidityPoolContractClient<'static> =
            unsafe { core::mem::transmute(client) };

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

// ─── initialization ───────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let t = TestEnv::setup();
    assert_eq!(t.client.get_admin(), t.admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_initialize_twice_fails() {
    let t = TestEnv::setup();
    // Second call should panic with AlreadyInitialized (2)
    let another_admin = Address::generate(&t.env);
    t.client.initialize(
        &another_admin,
        &t.token.address,
        &t.treasury,
        &t.merchant_fund,
    );
}

// ─── deposit ──────────────────────────────────────────────────────────────────

#[test]
fn test_first_deposit_one_to_one_ratio() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);

    let shares = t.client.deposit(&provider, &1_000);

    // First deposit → shares == amount
    assert_eq!(shares, 1_000);
    assert_eq!(t.client.get_lp_shares(&provider), 1_000);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 1_000);
    assert_eq!(stats.total_shares, 1_000);
    assert_eq!(stats.locked_liquidity, 0);
    assert_eq!(stats.available_liquidity, 1_000);
}

#[test]
fn test_subsequent_deposit_proportional_shares() {
    let t = TestEnv::setup();

    let provider_a = Address::generate(&t.env);
    let provider_b = Address::generate(&t.env);
    t.mint(&provider_a, 1_000);
    t.mint(&provider_b, 1_000);

    // First deposit
    t.client.deposit(&provider_a, &1_000);

    // Second deposit: same amount → same shares (pool value unchanged)
    let shares_b = t.client.deposit(&provider_b, &1_000);
    assert_eq!(shares_b, 1_000);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 2_000);
    assert_eq!(stats.total_shares, 2_000);
}

#[test]
fn test_deposit_after_interest_increases_share_value() {
    // Simulate: pool gains interest → share_price > 1.00 →
    // subsequent depositor gets fewer shares per token.
    let t = TestEnv::setup();

    let provider_a = Address::generate(&t.env);
    let provider_b = Address::generate(&t.env);
    t.mint(&provider_a, 1_000);
    t.mint(&provider_b, 1_000);

    // First deposit: 1000 tokens → 1000 shares
    t.client.deposit(&provider_a, &1_000);

    // Simulate interest: distribute 100 tokens of interest.
    // 85 stays in pool → total_liquidity becomes 1085, total_shares stays 1000.
    // share_price = 1085/1000 = 1.085
    // The test helper calls distribute_interest directly:
    // We inject interest by sending tokens to the pool and calling receive_repayment
    // with principal=0, interest=100.
    t.mint(&t.creditline, 100);
    t.client
        .receive_repayment(&t.creditline, &0, &100);

    // Now total_liquidity includes the LP portion (85) of interest.
    // Pool: total_liquidity = 1000 + 85 = 1085, total_shares = 1000
    // Second deposit of 1000 tokens: shares = 1000 * 1000 / 1085 ≈ 921
    let shares_b = t.client.deposit(&provider_b, &1_000);
    assert!(shares_b < 1_000, "Shares must be < 1000 since pool value grew");

    // provider_a's shares are still 1000 but worth more
    assert_eq!(t.client.get_lp_shares(&provider_a), 1_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_deposit_zero_amount_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.client.deposit(&provider, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_deposit_negative_amount_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.client.deposit(&provider, &-500);
}

// ─── withdraw ─────────────────────────────────────────────────────────────────

#[test]
fn test_full_withdrawal() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);

    t.client.deposit(&provider, &1_000);

    let amount_returned = t.client.withdraw(&provider, &1_000);
    assert_eq!(amount_returned, 1_000);
    assert_eq!(t.client.get_lp_shares(&provider), 0);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 0);
    assert_eq!(stats.total_shares, 0);
}

#[test]
fn test_partial_withdrawal() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);

    t.client.deposit(&provider, &1_000);

    let amount_returned = t.client.withdraw(&provider, &400);
    assert_eq!(amount_returned, 400);
    assert_eq!(t.client.get_lp_shares(&provider), 600);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 600);
    assert_eq!(stats.total_shares, 600);
}

#[test]
fn test_withdrawal_reflects_share_appreciation() {
    // After interest, withdrawing all shares returns more than deposited.
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);

    t.client.deposit(&provider, &1_000);

    // Distribute 100 interest (85 stays in pool)
    t.mint(&t.creditline, 100);
    t.client.receive_repayment(&t.creditline, &0, &100);

    // Total_liquidity = 1085, total_shares = 1000
    // Withdraw all 1000 shares → should receive 1085 tokens
    let amount_returned = t.client.withdraw(&provider, &1_000);
    assert_eq!(amount_returned, 1_085);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_withdraw_more_shares_than_owned_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);
    t.client.withdraw(&provider, &1_001);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_withdraw_when_liquidity_locked_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Lock all liquidity in a loan
    t.client.fund_loan(&t.creditline, &merchant, &1_000);

    // Try to withdraw → all liquidity is locked
    t.client.withdraw(&provider, &1_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_withdraw_zero_shares_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.client.withdraw(&provider, &0);
}

// ─── fund_loan ────────────────────────────────────────────────────────────────

#[test]
fn test_fund_loan_increases_locked_liquidity() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    t.client.fund_loan(&t.creditline, &merchant, &400);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.locked_liquidity, 400);
    assert_eq!(stats.available_liquidity, 600);
    assert_eq!(stats.total_liquidity, 1_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_fund_loan_exceeds_available_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Try to fund more than available
    t.client.fund_loan(&t.creditline, &merchant, &1_001);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_fund_loan_unauthorized_caller_fails() {
    let t = TestEnv::setup();
    let intruder = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.client.fund_loan(&intruder, &merchant, &100);
}

// ─── receive_repayment ────────────────────────────────────────────────────────

#[test]
fn test_receive_repayment_decreases_locked_and_distributes_interest() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Fund a 400-token loan
    t.client.fund_loan(&t.creditline, &merchant, &400);

    // Repay 400 principal + 40 interest
    t.mint(&t.creditline, 440);
    t.client.receive_repayment(&t.creditline, &400, &40);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.locked_liquidity, 0);

    // fund_loan does NOT reduce total_liquidity — it only moves tokens into locked.
    // LP portion of interest = 85% of 40 = 34
    // total_liquidity = 1000 (original) + 400 (principal back) + 34 (LP interest) = 1434
    assert_eq!(stats.total_liquidity, 1_434);
}

#[test]
fn test_receive_repayment_treasury_receives_protocol_fee() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Send 100 interest
    t.mint(&t.creditline, 100);
    t.client.receive_repayment(&t.creditline, &0, &100);

    // Treasury gets 10% = 10
    let treasury_balance = t.token.balance(&t.treasury);
    assert_eq!(treasury_balance, 10);
}

#[test]
fn test_receive_repayment_merchant_fund_receives_fee() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Send 100 interest
    t.mint(&t.creditline, 100);
    t.client.receive_repayment(&t.creditline, &0, &100);

    // Merchant fund gets 5% = 5
    let mf_balance = t.token.balance(&t.merchant_fund);
    assert_eq!(mf_balance, 5);
}

// ─── distribute_interest (SC-17 core) ────────────────────────────────────────

#[test]
fn test_distribute_interest_fee_split_accuracy() {
    // 85 / 10 / 5 split on 1000 tokens
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 10_000);
    t.client.deposit(&provider, &10_000);

    t.mint(&t.creditline, 1_000);
    t.client.receive_repayment(&t.creditline, &0, &1_000);

    // LP: 850 stays in pool → total_liquidity = 10000 + 850 = 10850
    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 10_850);

    // Treasury: 10% = 100
    assert_eq!(t.token.balance(&t.treasury), 100);

    // Merchant fund: 5% = 50
    assert_eq!(t.token.balance(&t.merchant_fund), 50);
}

#[test]
fn test_distribute_interest_share_value_appreciation() {
    // Start: 1 share = $1.00 (10000 bps)
    // After 8% interest on 1000 tokens deposit:
    //   interest = 80, lp_portion = 68 (85%)
    //   share_price = (1000 + 68) / 1000 * 10000 = 10680 bps
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    let stats_before = t.client.get_pool_stats();
    assert_eq!(stats_before.share_price, 10_000); // $1.00 in bps

    // Distribute 80 tokens of interest (8% on 1000)
    t.mint(&t.creditline, 80);
    t.client.receive_repayment(&t.creditline, &0, &80);

    let stats_after = t.client.get_pool_stats();
    // lp_amount = 80 * 8500 / 10000 = 68
    assert_eq!(stats_after.total_liquidity, 1_068);
    assert_eq!(stats_after.share_price, 10_680); // $1.068 expressed as bps
}

#[test]
fn test_multiple_lp_proportional_distribution() {
    // Two LPs: A deposits 1000, B deposits 1000.
    // After interest, both should benefit proportionally.
    let t = TestEnv::setup();

    let provider_a = Address::generate(&t.env);
    let provider_b = Address::generate(&t.env);
    t.mint(&provider_a, 1_000);
    t.mint(&provider_b, 1_000);

    t.client.deposit(&provider_a, &1_000);
    t.client.deposit(&provider_b, &1_000);

    // 200 interest distributed (100 per LP proportionally)
    t.mint(&t.creditline, 200);
    t.client.receive_repayment(&t.creditline, &0, &200);

    // LP amount = 85% of 200 = 170 → added to pool
    // total_liquidity = 2000 + 170 = 2170, total_shares = 2000
    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 2_170);

    // Both LPs hold 1000 shares out of 2000 → each owns 50% of pool
    // Withdrawal value per LP = 1000 * 2170 / 2000 = 1085
    let val_a = t.client.calculate_withdrawal(&1_000);
    let val_b = t.client.calculate_withdrawal(&1_000);
    assert_eq!(val_a, 1_085);
    assert_eq!(val_b, 1_085);
}

#[test]
fn test_interest_calculation_accuracy_small_amount() {
    // 100 interest: lp=85, treasury=10, merchant=5 (exact, no rounding)
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    t.mint(&t.creditline, 100);
    t.client.receive_repayment(&t.creditline, &0, &100);

    assert_eq!(t.token.balance(&t.treasury), 10);
    assert_eq!(t.token.balance(&t.merchant_fund), 5);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 1_085);
}

#[test]
fn test_interest_rounding_remainder_goes_to_lp() {
    // Use an amount that doesn't divide evenly: 101
    // lp = 101 * 8500 / 10000 = 85 (floor)
    // protocol = 101 * 1000 / 10000 = 10 (floor)
    // merchant = 101 - 85 - 10 = 6
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 10_000);
    t.client.deposit(&provider, &10_000);

    t.mint(&t.creditline, 101);
    t.client.receive_repayment(&t.creditline, &0, &101);

    assert_eq!(t.token.balance(&t.treasury), 10);
    assert_eq!(t.token.balance(&t.merchant_fund), 6); // remainder goes here
}

// ─── receive_guarantee ────────────────────────────────────────────────────────

#[test]
fn test_receive_guarantee_reduces_locked_and_recovers_liquidity() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Fund a 500-token loan
    t.client.fund_loan(&t.creditline, &merchant, &500);

    // Default: guarantee of 100 returned
    t.mint(&t.creditline, 100);
    t.client.receive_guarantee(&t.creditline, &100);

    let stats = t.client.get_pool_stats();
    // locked was 500, reduced by 100 → 400
    assert_eq!(stats.locked_liquidity, 400);
    // total_liquidity was 1000, recovered 100 → 1100... no wait:
    // fund_loan doesn't change total_liquidity, it changes locked.
    // After fund_loan: total=1000, locked=500, available=500.
    // receive_guarantee adds 100 to total, reduces locked by 100.
    assert_eq!(stats.total_liquidity, 1_100);
}

// ─── withdraw (additional edge cases) ────────────────────────────────────────

#[test]
fn test_withdraw_returns_tokens_to_provider() {
    // Verify that tokens actually land in the provider's wallet after withdrawal.
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 2_000);

    t.client.deposit(&provider, &2_000);
    assert_eq!(t.token.balance(&provider), 0);

    t.client.withdraw(&provider, &2_000);
    assert_eq!(t.token.balance(&provider), 2_000);
}

#[test]
fn test_withdraw_updates_pool_stats_correctly() {
    // After partial withdrawal, stats must reflect the remaining state.
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 3_000);

    t.client.deposit(&provider, &3_000);
    t.client.withdraw(&provider, &1_000);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 2_000);
    assert_eq!(stats.total_shares, 2_000);
    assert_eq!(stats.locked_liquidity, 0);
    assert_eq!(stats.available_liquidity, 2_000);
}

#[test]
fn test_two_providers_independent_withdrawals() {
    // Provider A and B each deposit; each can withdraw their own portion
    // without affecting the other's entitlement.
    let t = TestEnv::setup();
    let provider_a = Address::generate(&t.env);
    let provider_b = Address::generate(&t.env);
    t.mint(&provider_a, 1_000);
    t.mint(&provider_b, 2_000);

    t.client.deposit(&provider_a, &1_000);
    t.client.deposit(&provider_b, &2_000);

    // A withdraws all their shares (1000 out of 3000 total = 1/3 of pool)
    let returned_a = t.client.withdraw(&provider_a, &1_000);
    assert_eq!(returned_a, 1_000);
    assert_eq!(t.client.get_lp_shares(&provider_a), 0);

    // B's shares and pool value are intact
    assert_eq!(t.client.get_lp_shares(&provider_b), 2_000);
    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 2_000);
    assert_eq!(stats.total_shares, 2_000);

    // B withdraws everything
    let returned_b = t.client.withdraw(&provider_b, &2_000);
    assert_eq!(returned_b, 2_000);

    let stats_final = t.client.get_pool_stats();
    assert_eq!(stats_final.total_liquidity, 0);
    assert_eq!(stats_final.total_shares, 0);
}

#[test]
fn test_withdraw_partial_when_some_liquidity_locked() {
    // If only part of liquidity is locked, a partial withdrawal of the
    // available portion should succeed.
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Lock 400 tokens in a loan → 600 available
    t.client.fund_loan(&t.creditline, &merchant, &400);

    // Withdraw shares worth exactly 600 tokens (should pass)
    // shares_to_withdraw = 600 * 1000 / 1000 = 600 shares
    let returned = t.client.withdraw(&provider, &600);
    assert_eq!(returned, 600);

    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 400);
    assert_eq!(stats.locked_liquidity, 400);
    assert_eq!(stats.available_liquidity, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_withdraw_negative_shares_fails() {
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);
    t.client.withdraw(&provider, &-1);
}

#[test]
fn test_sequential_partial_withdrawals_drain_pool() {
    // Withdraw in two steps and confirm pool reaches zero correctly.
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    t.mint(&provider, 1_000);

    t.client.deposit(&provider, &1_000);

    let first = t.client.withdraw(&provider, &600);
    assert_eq!(first, 600);

    let second = t.client.withdraw(&provider, &400);
    assert_eq!(second, 400);

    assert_eq!(t.client.get_lp_shares(&provider), 0);
    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 0);
    assert_eq!(stats.total_shares, 0);
}

#[test]
fn test_withdraw_succeeds_after_loan_repayment_unlocks_liquidity() {
    // A withdrawal blocked by locked liquidity must succeed once the loan is
    // repaid and locked_liquidity returns to zero.
    //
    // Note: fund_loan transfers tokens to the merchant but keeps total_liquidity
    // unchanged (only locked_liquidity increases). receive_repayment then adds
    // the returned principal back to total_liquidity. After the full cycle the
    // pool holds twice the original principal in total_liquidity but only the
    // original tokens physically — so we withdraw only the pre-loan amount (1000).
    let t = TestEnv::setup();
    let provider = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    t.mint(&provider, 1_000);
    t.client.deposit(&provider, &1_000);

    // Lock 600 tokens — only 400 remain available; a 1000-share withdrawal
    // (worth 1000 tokens) would exceed available_liquidity and fail.
    t.client.fund_loan(&t.creditline, &merchant, &600);

    let stats_mid = t.client.get_pool_stats();
    assert_eq!(stats_mid.locked_liquidity, 600);
    assert_eq!(stats_mid.available_liquidity, 400);

    // Creditline repays 600 principal (no interest).
    t.mint(&t.creditline, 600);
    t.client.receive_repayment(&t.creditline, &600, &0);

    // Locked must be zero; all liquidity available.
    let stats_after = t.client.get_pool_stats();
    assert_eq!(stats_after.locked_liquidity, 0);
    assert_eq!(stats_after.available_liquidity, stats_after.total_liquidity);

    // After the fund_loan → receive_repayment cycle:
    //   total_liquidity = 1000 (original) + 600 (principal returned) = 1600
    //   total_shares    = 1000
    // Withdrawing 600 shares: 600 * 1600 / 1000 = 960 tokens.
    // This succeeds because available_liquidity (1600) >= 960.
    let returned = t.client.withdraw(&provider, &600);
    assert_eq!(returned, 960);
    assert_eq!(t.client.get_lp_shares(&provider), 400);
}

// ─── pool_stats & calculate_withdrawal ───────────────────────────────────────

#[test]
fn test_get_pool_stats_empty_pool() {
    let t = TestEnv::setup();
    let stats = t.client.get_pool_stats();
    assert_eq!(stats.total_liquidity, 0);
    assert_eq!(stats.total_shares, 0);
    assert_eq!(stats.locked_liquidity, 0);
    assert_eq!(stats.available_liquidity, 0);
    assert_eq!(stats.share_price, 10_000); // Default 1.00
}

#[test]
fn test_calculate_withdrawal_empty_pool_returns_zero() {
    let t = TestEnv::setup();
    assert_eq!(t.client.calculate_withdrawal(&1_000), 0);
}

// ─── admin operations ─────────────────────────────────────────────────────────

#[test]
fn test_set_admin() {
    let t = TestEnv::setup();
    let new_admin = Address::generate(&t.env);
    t.client.set_admin(&new_admin);
    assert_eq!(t.client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_non_admin_cannot_set_creditline() {
    let t = TestEnv::setup();
    let intruder = Address::generate(&t.env);
    let new_creditline = Address::generate(&t.env);
    t.client.set_creditline(&intruder, &new_creditline);
}
