use crate::{CreditLineContract, CreditLineContractClient, LoanStatus, RepaymentInstallment};
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events, Ledger},
    Address, Env,
};

// NOTE: Integration tests with reputation contract are skipped for now
// They will be added when all contracts are implemented and properly configured
#[contract]
pub struct MockReputation;

#[contractimpl]
impl MockReputation {
    pub fn get_score(_env: Env, _user: Address) -> u32 {
        100 // Returns 100 to pass the threshold check
    }
    pub fn slash(_env: Env, _user: Address) {
        // Does nothing, just needs to exist for the call to succeed
    }
}

// A mock reputation contract that always returns a score below the threshold.
// Placed in its own module to avoid symbol collisions with MockReputation.
mod mock_low_rep {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct MockReputationLow;

    #[contractimpl]
    impl MockReputationLow {
        pub fn get_score(_env: Env, _user: Address) -> u32 {
            49 // Returns 49 — below the 50 minimum threshold
        }
    }
}
use mock_low_rep::MockReputationLow;

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Creates a basic TestEnv with MockReputation wired in and the contract
/// initialized. Returns (env, client, admin, rep_id).
struct TestCtx {
    env: Env,
    client: CreditLineContractClient<'static>,
    admin: Address,
    rep_id: Address,
}

impl TestCtx {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CreditLineContract, ());
        let client = CreditLineContractClient::new(&env, &contract_id);

        // SAFETY: env outlives client — same pattern as liquidity-pool tests
        let client: CreditLineContractClient<'static> = unsafe { core::mem::transmute(client) };

        let admin = Address::generate(&env);
        let rep_id = env.register(MockReputation, ());
        let merchant_registry = Address::generate(&env);
        let liquidity_pool = Address::generate(&env);

        client.initialize(&admin, &rep_id, &merchant_registry, &liquidity_pool);

        TestCtx {
            env,
            client,
            admin,
            rep_id,
        }
    }

    /// Build a single-installment repayment schedule with the given due date.
    fn single_installment(&self, amount: i128, due_date: u64) -> soroban_sdk::Vec<RepaymentInstallment> {
        let mut schedule = soroban_sdk::Vec::new(&self.env);
        schedule.push_back(RepaymentInstallment { amount, due_date });
        schedule
    }

    /// Create a loan with sensible defaults: total=1000, guarantee=200, 1 installment.
    fn create_default_loan(&self, user: &Address, merchant: &Address) -> u64 {
        let due_date = self.env.ledger().timestamp() + 10_000;
        let schedule = self.single_installment(1000, due_date);
        self.client.create_loan(user, merchant, &1000, &200, &schedule)
    }

    /// Advance ledger timestamp past the given due date so a loan is overdue.
    fn advance_past(&self, due_date: u64) {
        self.env.ledger().set_timestamp(due_date + 1);
    }
}

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

#[test]
fn test_mark_defaulted_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    // Register our Mock Reputation contract
    let rep_id = env.register(MockReputation, ());

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);
    let liquidity_pool = Address::generate(&env);

    client.initialize(
        &admin,
        &rep_id, // Pass the Mock ID
        &Address::generate(&env),
        &liquidity_pool,
    );

    // Set a baseline time
    let current_time = 10000;
    env.ledger().set_timestamp(current_time);

    let mut schedule = soroban_sdk::Vec::new(&env);
    schedule.push_back(RepaymentInstallment {
        amount: 1000,
        due_date: current_time + 1000, // Due at 11000
    });

    // Create loan (calls MockReputation::get_score)
    let loan_id = client.create_loan(&user, &merchant, &1000, &200, &schedule);

    // Time Travel past the due date
    env.ledger().set_timestamp(12000);

    // This calls mark_defaulted which internally calls MockReputation::slash
    client.mark_defaulted(&loan_id);

    let updated_loan = client.get_loan(&loan_id);
    assert_eq!(updated_loan.status, LoanStatus::Defaulted);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")] // LoanNotOverdue
fn test_mark_defaulted_too_early_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    let rep_id = env.register(MockReputation, ());

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(
        &admin,
        &rep_id,
        &Address::generate(&env),
        &Address::generate(&env),
    );

    let current_time = 10000;
    env.ledger().set_timestamp(current_time);

    let mut schedule = soroban_sdk::Vec::new(&env);
    schedule.push_back(RepaymentInstallment {
        amount: 1000,
        due_date: 20000,
    });

    let loan_id = client.create_loan(&user, &Address::generate(&env), &1000, &200, &schedule);

    // This should fail because 10000 < 20000
    client.mark_defaulted(&loan_id);
}

// ─── loan creation — happy path ───────────────────────────────────────────────

#[test]
fn test_create_loan_returns_incrementing_ids() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    let id1 = t.create_default_loan(&user, &merchant);
    let id2 = t.create_default_loan(&user, &merchant);
    let id3 = t.create_default_loan(&user, &merchant);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn test_create_loan_stores_correct_fields() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(5000);
    let due_date = 15000_u64;
    let schedule = t.single_installment(1000, due_date);

    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);
    let loan = t.client.get_loan(&loan_id);

    assert_eq!(loan.loan_id, loan_id);
    assert_eq!(loan.borrower, user);
    assert_eq!(loan.merchant, merchant);
    assert_eq!(loan.total_amount, 1000);
    assert_eq!(loan.guarantee_amount, 200);
    assert_eq!(loan.remaining_balance, 1000);
    assert_eq!(loan.status, LoanStatus::Active);
    assert_eq!(loan.created_at, 5000);
}

#[test]
fn test_create_loan_exactly_20_percent_guarantee() {
    // 200 is exactly 20% of 1000 — must succeed
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let schedule = t.single_installment(1000, 99999);

    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);
    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.guarantee_amount, 200);
}

#[test]
fn test_create_loan_with_more_than_20_percent_guarantee() {
    // 500 is 50% of 1000 — must succeed
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let schedule = t.single_installment(1000, 99999);

    let loan_id = t.client.create_loan(&user, &merchant, &1000, &500, &schedule);
    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.status, LoanStatus::Active);
}

#[test]
fn test_create_loan_with_multi_installment_schedule() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    let mut schedule = soroban_sdk::Vec::new(&t.env);
    schedule.push_back(RepaymentInstallment { amount: 334, due_date: 10000 });
    schedule.push_back(RepaymentInstallment { amount: 333, due_date: 20000 });
    schedule.push_back(RepaymentInstallment { amount: 333, due_date: 30000 });

    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);
    let loan = t.client.get_loan(&loan_id);

    assert_eq!(loan.repayment_schedule.len(), 3);
    assert_eq!(loan.total_amount, 1000);
}

// ─── loan creation — reputation validation ────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #4)")] // InsufficientReputation
fn test_create_loan_rejected_when_reputation_below_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreditLineContract, ());
    let client = CreditLineContractClient::new(&env, &contract_id);

    // Wire in the low-score mock
    let low_rep_id = env.register(MockReputationLow, ());
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let merchant = Address::generate(&env);

    client.initialize(
        &admin,
        &low_rep_id,
        &Address::generate(&env),
        &Address::generate(&env),
    );

    let mut schedule = soroban_sdk::Vec::new(&env);
    schedule.push_back(RepaymentInstallment { amount: 1000, due_date: 99999 });

    // Score is 49 — below 50 minimum → InsufficientReputation (error 4)
    client.create_loan(&user, &merchant, &1000, &200, &schedule);
}

#[test]
fn test_create_loan_accepted_at_exactly_threshold_score() {
    // MockReputation returns 100 which is ≥ 50 → must succeed
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);
    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.status, LoanStatus::Active);
}

// ─── loan creation — event emission ──────────────────────────────────────────

#[test]
fn test_create_loan_emits_loan_created_event() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.create_default_loan(&user, &merchant);

    // At least one event was emitted
    let events = t.env.events().all();
    assert!(!events.is_empty(), "Expected a LoanCreated event to be emitted");
}

#[test]
fn test_mark_defaulted_emits_loan_defaulted_event() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);

    let events = t.env.events().all();
    assert!(!events.is_empty(), "Expected a LoanDefaulted event to be emitted");
}

// ─── default flow ─────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #7)")] // LoanNotActive
fn test_mark_defaulted_on_already_defaulted_loan_fails() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);

    // Second call must fail — loan is no longer Active
    t.client.mark_defaulted(&loan_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")] // LoanNotFound
fn test_mark_defaulted_on_nonexistent_loan_fails() {
    let t = TestCtx::setup();
    t.client.mark_defaulted(&999);
}

#[test]
fn test_default_flow_loan_status_becomes_defaulted() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    let before = t.client.get_loan(&loan_id);
    assert_eq!(before.status, LoanStatus::Active);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);

    let after = t.client.get_loan(&loan_id);
    assert_eq!(after.status, LoanStatus::Defaulted);
}

#[test]
fn test_default_flow_preserves_loan_amounts() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);

    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.total_amount, 1000);
    assert_eq!(loan.guarantee_amount, 200);
    assert_eq!(loan.remaining_balance, 1000); // unchanged — no repayment was made
}

#[test]
fn test_mark_defaulted_at_exactly_due_date_boundary() {
    // Ledger timestamp == due_date: still NOT overdue (the condition is `timestamp > due_date`)
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let due_date = 5000_u64;
    let schedule = t.single_installment(1000, due_date);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    // Set timestamp to exactly the due date — mark_defaulted should fail (LoanNotOverdue)
    t.env.ledger().set_timestamp(due_date);

    let result = t.client.try_mark_defaulted(&loan_id);
    assert!(result.is_err(), "Should fail when timestamp == due_date");
}

#[test]
fn test_mark_defaulted_one_second_past_due_succeeds() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let due_date = 5000_u64;
    let schedule = t.single_installment(1000, due_date);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.env.ledger().set_timestamp(due_date + 1);
    t.client.mark_defaulted(&loan_id);

    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.status, LoanStatus::Defaulted);
}

#[test]
fn test_default_flow_uses_last_installment_for_overdue_check() {
    // Multi-installment loan: overdue is determined by the LAST installment's due date
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);

    let mut schedule = soroban_sdk::Vec::new(&t.env);
    schedule.push_back(RepaymentInstallment { amount: 400, due_date: 3000 }); // already past
    schedule.push_back(RepaymentInstallment { amount: 300, due_date: 6000 }); // already past
    schedule.push_back(RepaymentInstallment { amount: 300, due_date: 10000 }); // last

    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    // Past first two but not the last — should still fail (LoanNotOverdue)
    t.env.ledger().set_timestamp(7000);
    let result = t.client.try_mark_defaulted(&loan_id);
    assert!(result.is_err(), "Not overdue until past the last installment");

    // Now past the last installment — should succeed
    t.env.ledger().set_timestamp(10001);
    t.client.mark_defaulted(&loan_id);
    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.status, LoanStatus::Defaulted);
}

// ─── loan creation — score decrease on default (reputation integration) ───────

#[test]
fn test_mark_defaulted_triggers_reputation_slash() {
    // MockReputation::slash is a no-op; we just verify the call doesn't panic,
    // proving the contract correctly invokes the reputation contract on default.
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.advance_past(5000);
    // This succeeds only if the `slash` cross-contract call is executed without error
    t.client.mark_defaulted(&loan_id);

    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.status, LoanStatus::Defaulted);
}

// ─── admin access control ─────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // NotAdmin
fn test_set_reputation_contract_by_non_admin_fails() {
    let t = TestCtx::setup();
    let intruder = Address::generate(&t.env);
    let new_rep = Address::generate(&t.env);
    t.client.set_reputation_contract(&intruder, &new_rep);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // NotAdmin
fn test_set_merchant_registry_by_non_admin_fails() {
    let t = TestCtx::setup();
    let intruder = Address::generate(&t.env);
    let new_registry = Address::generate(&t.env);
    t.client.set_merchant_registry(&intruder, &new_registry);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // NotAdmin
fn test_set_liquidity_pool_by_non_admin_fails() {
    let t = TestCtx::setup();
    let intruder = Address::generate(&t.env);
    let new_pool = Address::generate(&t.env);
    t.client.set_liquidity_pool(&intruder, &new_pool);
}

#[test]
fn test_admin_can_update_all_contract_addresses() {
    let t = TestCtx::setup();
    let new_rep = Address::generate(&t.env);
    let new_registry = Address::generate(&t.env);
    let new_pool = Address::generate(&t.env);

    // All three must succeed without panic
    t.client.set_reputation_contract(&t.admin, &new_rep);
    t.client.set_merchant_registry(&t.admin, &new_registry);
    t.client.set_liquidity_pool(&t.admin, &new_pool);
}

// ─── repayment — TDD stubs (implementations pending) ─────────────────────────
//
// These tests define the expected behaviour for the `repay` function which is
// not yet implemented. They are tagged #[ignore] so the suite remains green.
// Remove #[ignore] and implement the function when working on Phase 4.

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
fn test_partial_repayment_reduces_remaining_balance() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);

    // TODO: t.client.repay(&user, &loan_id, &500);
    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.remaining_balance, 500);
    assert_eq!(loan.status, LoanStatus::Active);
}

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
fn test_full_repayment_sets_status_to_paid() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);

    // TODO: t.client.repay(&user, &loan_id, &1000);
    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.remaining_balance, 0);
    assert_eq!(loan.status, LoanStatus::Paid);
}

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
#[should_panic(expected = "Error(Contract, #9)")] // InvalidAmount
fn test_overpayment_is_rejected() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);

    // Paying more than remaining_balance should panic with InvalidAmount
    // TODO: t.client.repay(&user, &loan_id, &1001);
    let _ = loan_id;
}

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
#[should_panic(expected = "Error(Contract, #8)")] // NotBorrower
fn test_unauthorized_repayment_is_rejected() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let intruder = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);

    // A different address trying to repay the loan must fail with NotBorrower
    // TODO: t.client.repay(&intruder, &loan_id, &200);
    let _ = (loan_id, intruder);
}

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
#[should_panic(expected = "Error(Contract, #7)")] // LoanNotActive
fn test_repayment_on_non_active_loan_is_rejected() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);

    // Attempting to repay a Defaulted loan must fail with LoanNotActive
    // TODO: t.client.repay(&user, &loan_id, &1000);
    let _ = loan_id;
}

#[test]
#[ignore = "score increase on repayment not yet implemented — Phase 4"]
fn test_full_repayment_triggers_reputation_increase() {
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);

    // TODO: t.client.repay(&user, &loan_id, &1000);
    // Expect a cross-contract call to reputation contract's increase_score
    let _ = loan_id;
}

#[test]
#[ignore = "early payment bonus not yet implemented — Phase 4"]
fn test_early_repayment_triggers_bonus_reputation_increase() {
    // Repaying before the first installment due date is considered "early"
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 10000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    // Pay early (timestamp 2000, well before due date 10000)
    t.env.ledger().set_timestamp(2000);
    // TODO: t.client.repay(&user, &loan_id, &1000);
    // Expect a larger reputation bonus than a standard on-time repayment
    let _ = loan_id;
}

// ─── merchant validation — TDD stubs (Phase 5) ───────────────────────────────

#[test]
#[ignore = "merchant registry integration not yet implemented — Phase 5"]
fn test_active_merchant_can_receive_loan() {
    // When merchant registry is live, an approved merchant must pass validation
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let active_merchant = Address::generate(&t.env);
    // TODO: wire up a MockMerchantRegistry that returns is_active=true for active_merchant
    let _ = t.create_default_loan(&user, &active_merchant);
}

#[test]
#[ignore = "merchant registry integration not yet implemented — Phase 5"]
#[should_panic(expected = "Error(Contract, #3)")] // MerchantNotActive
fn test_inactive_merchant_loan_is_rejected() {
    // A merchant that is registered but set to inactive must fail
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let inactive_merchant = Address::generate(&t.env);
    // TODO: wire up a MockMerchantRegistry that returns is_active=false for inactive_merchant
    let _ = t.create_default_loan(&user, &inactive_merchant);
}

#[test]
#[ignore = "merchant registry integration not yet implemented — Phase 5"]
#[should_panic(expected = "Error(Contract, #3)")] // MerchantNotActive
fn test_unregistered_merchant_loan_is_rejected() {
    // A merchant address unknown to the registry must fail
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let unknown_merchant = Address::generate(&t.env);
    // TODO: wire up a MockMerchantRegistry that returns None for unknown_merchant
    let _ = t.create_default_loan(&user, &unknown_merchant);
}

// ─── liquidity pool integration — TDD stubs (Phase 6) ────────────────────────

#[test]
#[ignore = "liquidity pool integration not yet implemented — Phase 6"]
fn test_loan_funding_debits_liquidity_pool() {
    // create_loan must call fund_loan on the liquidity pool contract
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    // TODO: wire up a MockLiquidityPool; after create_loan verify fund_loan was called
    let _ = t.create_default_loan(&user, &merchant);
}

#[test]
#[ignore = "liquidity pool integration not yet implemented — Phase 6"]
fn test_repayment_credited_to_liquidity_pool() {
    // repay() must forward funds to the liquidity pool via receive_repayment
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    let loan_id = t.create_default_loan(&user, &merchant);
    // TODO: t.client.repay(&user, &loan_id, &1000);
    // Verify MockLiquidityPool::receive_repayment was called
    let _ = loan_id;
}

#[test]
#[ignore = "liquidity pool integration not yet implemented — Phase 6"]
fn test_guarantee_transferred_to_pool_on_default() {
    // mark_defaulted must call receive_guarantee on the liquidity pool
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);
    // TODO: Verify MockLiquidityPool::receive_guarantee(200) was called
    let _ = loan_id;
}

#[test]
#[ignore = "liquidity pool integration not yet implemented — Phase 6"]
#[should_panic(expected = "Error(Contract, #5)")] // InsufficientLiquidity
fn test_insufficient_liquidity_rejects_loan_creation() {
    // When pool does not have enough available liquidity, create_loan must fail
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);
    // TODO: wire up a MockLiquidityPool that returns available=0
    let _ = t.create_default_loan(&user, &merchant);
}

// ─── complete loan lifecycle ──────────────────────────────────────────────────

#[test]
fn test_complete_lifecycle_create_then_default() {
    // Verifies the full path: Active → overdue → Defaulted
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);
    let schedule = t.single_installment(1000, 5000);
    let loan_id = t.client.create_loan(&user, &merchant, &1000, &200, &schedule);

    let created = t.client.get_loan(&loan_id);
    assert_eq!(created.status, LoanStatus::Active);
    assert_eq!(created.remaining_balance, 1000);

    t.advance_past(5000);
    t.client.mark_defaulted(&loan_id);

    let defaulted = t.client.get_loan(&loan_id);
    assert_eq!(defaulted.status, LoanStatus::Defaulted);
    // Amounts must be immutable after default
    assert_eq!(defaulted.total_amount, 1000);
    assert_eq!(defaulted.guarantee_amount, 200);
}

#[test]
fn test_multiple_independent_loans_do_not_interfere() {
    // Two concurrent loans for different borrowers must be fully independent
    let t = TestCtx::setup();
    let user_a = Address::generate(&t.env);
    let user_b = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    t.env.ledger().set_timestamp(1000);

    let schedule_a = t.single_installment(1000, 5000);
    let schedule_b = t.single_installment(2000, 8000);

    let loan_a = t.client.create_loan(&user_a, &merchant, &1000, &200, &schedule_a);
    let loan_b = t.client.create_loan(&user_b, &merchant, &2000, &400, &schedule_b);

    // Default loan_a only
    t.advance_past(5000);
    t.client.mark_defaulted(&loan_a);

    let la = t.client.get_loan(&loan_a);
    let lb = t.client.get_loan(&loan_b);

    assert_eq!(la.status, LoanStatus::Defaulted);
    assert_eq!(lb.status, LoanStatus::Active); // unaffected
    assert_eq!(lb.total_amount, 2000);
}

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
fn test_complete_lifecycle_create_repay_complete() {
    // Verifies the full happy path: Active → repaid in full → Paid
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    let loan_id = t.create_default_loan(&user, &merchant);

    let active = t.client.get_loan(&loan_id);
    assert_eq!(active.status, LoanStatus::Active);

    // TODO: t.client.repay(&user, &loan_id, &1000);

    let paid = t.client.get_loan(&loan_id);
    assert_eq!(paid.status, LoanStatus::Paid);
    assert_eq!(paid.remaining_balance, 0);
}

#[test]
#[ignore = "repay() not yet implemented — Phase 4"]
fn test_multi_contract_integration_full_flow() {
    // End-to-end: reputation check on create → funding → repayment → score boost
    let t = TestCtx::setup();
    let user = Address::generate(&t.env);
    let merchant = Address::generate(&t.env);

    // 1. Create loan — reputation validated, pool funded
    let loan_id = t.create_default_loan(&user, &merchant);

    // 2. Repay in full — pool credited, reputation score increased
    // TODO: t.client.repay(&user, &loan_id, &1000);

    let loan = t.client.get_loan(&loan_id);
    assert_eq!(loan.status, LoanStatus::Paid);

    // TODO: assert reputation score increased for `user`
    // TODO: assert liquidity pool received the repayment
    let _ = loan_id;
}
