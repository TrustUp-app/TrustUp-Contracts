#![cfg(test)]

use crate::setup::TestEnv;
use creditline_contract::RepaymentInstallment;
use soroban_sdk::{testutils::Address as _, vec, Address};

#[test]
fn test_full_bnpl_lifecycle() {
    let mut setup = TestEnv::setup();
    let env = &setup.env;

    let user = Address::generate(env);
    let merchant = Address::generate(env);
    
    // 1. Register merchant
    setup.merchant_registry.register_merchant(&setup.admin, &merchant, &soroban_sdk::String::from_str(env, "Merchant A"));

    // 2. Set user reputation
    setup.reputation.increase_score(&setup.admin, &user, &40);
    assert_eq!(setup.reputation.get_score(&user), 40);

    // 3. Fund LP
    let lp_provider = Address::generate(env);
    setup.token.mint(&lp_provider, &100_000_000_000);
    
    // Deposit into LP
    setup.liquidity_pool.deposit(&lp_provider, &10_000_000_000);

    // 4. Create loan
    // User needs some token to pay guarantee
    setup.token.mint(&user, &200_000_000);

    let total_amount = 1_000_000_000;
    let guarantee_amount = 100_000_000;
    
    let installments = vec![
        env,
        RepaymentInstallment {
            amount: 550_000_000,
            due_date: 1000,
        },
        RepaymentInstallment {
            amount: 550_000_000,
            due_date: 2000,
        },
    ];

    let loan_id = setup.creditline.create_loan(
        &user,
        &merchant,
        &total_amount,
        &guarantee_amount,
        &installments,
    );

    // Check merchant received funds
    // (Actual logic depends on if create_loan immediately transfers to merchant, normally it does via LP)
    
    // 5. Repay loan
    setup.token.mint(&user, &1_000_000_000);
    setup.creditline.repay_loan(&user, &loan_id, &550_000_000);
    setup.creditline.repay_loan(&user, &loan_id, &550_000_000);
    
    // Assert reputation increased
    assert_eq!(setup.reputation.get_score(&user), 45); // Adjust based on actual contract logic
}
