#![cfg(test)]

use crate::setup::TestEnv;
use creditline_contract::{LoanStatus, RepaymentInstallment};
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, vec, Address};
use std::panic::AssertUnwindSafe;

/// Representing discrete pool & credit line operations for invariant simulation
#[derive(Clone, Debug)]
enum Action {
    Deposit {
        provider_idx: usize,
        amount: i128,
    },
    Withdraw {
        provider_idx: usize,
        share_ratio_bps: u32,
    },
    CreateLoan {
        user_idx: usize,
        merchant_idx: usize,
        total_amount: i128,
        guarantee_percent: u32,
    },
    RepayLoan {
        loan_idx: usize,
        repay_percent: u32,
    },
}

fn prop_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        // Deposits: between MIN_AMOUNT (1) and 1,000,000,000
        (0..3usize, 1..1_000_000_000i128).prop_map(|(provider_idx, amount)| {
            Action::Deposit {
                provider_idx,
                amount,
            }
        }),
        // Withdrawals: 1% to 100% of provider shares (100 to 10000 bps)
        (0..3usize, 100..=10000u32).prop_map(|(provider_idx, share_ratio_bps)| {
            Action::Withdraw {
                provider_idx,
                share_ratio_bps,
            }
        }),
        // Create Loan: amount between 100 and 1,000, guarantee 20% to 50%
        (0..3usize, 0..2usize, 100..1_000i128, 20..=50u32).prop_map(
            |(user_idx, merchant_idx, total_amount, guarantee_percent)| {
                Action::CreateLoan {
                    user_idx,
                    merchant_idx,
                    total_amount,
                    guarantee_percent,
                }
            }
        ),
        // Repay Loan: 10% to 100% of remaining balance
        (0..10usize, 10..=100u32).prop_map(|(loan_idx, repay_percent)| Action::RepayLoan {
            loan_idx,
            repay_percent,
        }),
    ]
}

fn assert_invariants(setup: &TestEnv, providers: &[Address], active_loans: &[u64]) {
    let lp_stats = setup.liquidity_pool.get_pool_stats();

    // -------------------------------------------------------------------------
    // Invariant 1: total_shares == Σ(provider_shares) in liquidity-pool-contract
    // -------------------------------------------------------------------------
    let sum_provider_shares: i128 = providers
        .iter()
        .map(|p| setup.liquidity_pool.get_lp_shares(p))
        .sum();

    assert_eq!(
        lp_stats.total_shares, sum_provider_shares,
        "Invariant 1 Violated: total_shares ({}) != sum of provider_shares ({})",
        lp_stats.total_shares, sum_provider_shares
    );

    // -------------------------------------------------------------------------
    // Invariant 2: available_liquidity + locked_liquidity == total_liquidity
    // -------------------------------------------------------------------------
    let calculated_total = lp_stats
        .available_liquidity
        .checked_add(lp_stats.locked_liquidity)
        .expect("Overflow calculating available + locked liquidity");

    assert_eq!(
        calculated_total, lp_stats.total_liquidity,
        "Invariant 2 Violated: available ({}) + locked ({}) != total ({})",
        lp_stats.available_liquidity, lp_stats.locked_liquidity, lp_stats.total_liquidity
    );

    // -------------------------------------------------------------------------
    // Invariant 3: Σ(pool contribution portion of active loan balance) == locked_liquidity
    // -------------------------------------------------------------------------
    let mut sum_active_pool_locked = 0i128;
    for &loan_id in active_loans {
        if let Ok(loan) =
            std::panic::catch_unwind(AssertUnwindSafe(|| setup.creditline.get_loan(&loan_id)))
        {
            if loan.status == LoanStatus::Active {
                let pool_locked_for_loan = loan
                    .principal_outstanding
                    .saturating_sub(loan.guarantee_amount);
                sum_active_pool_locked += pool_locked_for_loan;
            }
        }
    }

    assert_eq!(
        lp_stats.locked_liquidity, sum_active_pool_locked,
        "Invariant 3 Violated: locked_liquidity ({}) != sum active pool locked ({})",
        lp_stats.locked_liquidity, sum_active_pool_locked
    );

    // -------------------------------------------------------------------------
    // Invariant 4: underlying token balance reconciles with pool-internal available_liquidity
    // -------------------------------------------------------------------------
    let pool_token_balance = setup.token.balance(&setup.liquidity_pool.address);
    assert_eq!(
        pool_token_balance, lp_stats.available_liquidity,
        "Invariant 4 Violated: pool token balance ({}) != available_liquidity ({})",
        pool_token_balance, lp_stats.available_liquidity
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_property_invariants_random_sequences(actions in prop::collection::vec(prop_action_strategy(), 1..25)) {
        let setup = TestEnv::setup();
        let env = &setup.env;

        let providers = vec![env, Address::generate(env), Address::generate(env), Address::generate(env)];
        let merchants = vec![env, Address::generate(env), Address::generate(env)];
        let users = vec![env, Address::generate(env), Address::generate(env), Address::generate(env)];

        // Register merchants and set user reputation
        for (i, merchant) in merchants.iter().enumerate() {
            let name_str = match i {
                0 => "Merchant Alpha",
                1 => "Merchant Beta",
                _ => "Merchant Gamma",
            };
            setup.merchant_registry.register_merchant(
                &setup.admin,
                &merchant,
                &soroban_sdk::String::from_str(env, name_str),
            );
        }

        for user in users.iter() {
            setup.reputation.increase_score(&setup.admin, &user, &90);
        }

        // Mint initial tokens for providers and users
        for provider in providers.iter() {
            setup.token_admin_client.mint(&provider, &10_000_000_000);
        }
        for user in users.iter() {
            setup.token_admin_client.mint(&user, &10_000_000_000);
        }

        let mut provider_addrs = Vec::new();
        for p in providers.iter() {
            provider_addrs.push(p);
        }

        let mut created_loans: std::vec::Vec<u64> = std::vec::Vec::new();

        // Initial invariant state check
        assert_invariants(&setup, &provider_addrs, &created_loans);

        for action in actions {
            match action {
                Action::Deposit { provider_idx, amount } => {
                    let provider = &provider_addrs[provider_idx % provider_addrs.len()];
                    let current_balance = setup.token.balance(provider);
                    if current_balance >= amount && amount >= 1 {
                        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                            setup.liquidity_pool.deposit(provider, &amount)
                        }));
                    }
                }

                Action::Withdraw { provider_idx, share_ratio_bps } => {
                    let provider = &provider_addrs[provider_idx % provider_addrs.len()];
                    let provider_shares = setup.liquidity_pool.get_lp_shares(provider);
                    if provider_shares > 0 {
                        let shares_to_withdraw = (provider_shares * share_ratio_bps as i128) / 10000;
                        if shares_to_withdraw > 0 {
                            let lp_stats = setup.liquidity_pool.get_pool_stats();
                            if lp_stats.total_liquidity > 0 {
                                let max_withdrawable_shares = (lp_stats.available_liquidity * lp_stats.total_shares) / lp_stats.total_liquidity;
                                let safe_shares = shares_to_withdraw.min(max_withdrawable_shares);
                                if safe_shares > 0 {
                                    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                                        setup.liquidity_pool.withdraw(provider, &safe_shares)
                                    }));
                                }
                            }
                        }
                    }
                }

                Action::CreateLoan { user_idx, merchant_idx, total_amount, guarantee_percent } => {
                    let user_idx_u32 = (user_idx as u32) % users.len();
                    let merchant_idx_u32 = (merchant_idx as u32) % merchants.len();
                    let user = users.get(user_idx_u32).unwrap();
                    let merchant = merchants.get(merchant_idx_u32).unwrap();

                    let guarantee_amount = (total_amount * guarantee_percent as i128) / 100;
                    let pool_contribution = total_amount - guarantee_amount;

                    let lp_stats = setup.liquidity_pool.get_pool_stats();
                    let user_token_bal = setup.token.balance(&user);

                    if lp_stats.available_liquidity >= pool_contribution
                        && user_token_bal >= guarantee_amount
                        && guarantee_amount >= (total_amount * 20) / 100
                        && total_amount > 0
                    {
                        let installments = vec![
                            env,
                            RepaymentInstallment {
                                amount: total_amount / 2,
                                due_date: 1000,
                            },
                            RepaymentInstallment {
                                amount: total_amount / 2,
                                due_date: 2000,
                            },
                        ];

                        let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
                            setup.creditline.create_loan(
                                &user,
                                &merchant,
                                &total_amount,
                                &guarantee_amount,
                                &installments,
                            )
                        }));
                        if let Ok(loan_id) = res {
                            created_loans.push(loan_id);
                        }
                    }
                }

                Action::RepayLoan { loan_idx, repay_percent } => {
                    if !created_loans.is_empty() {
                        let loan_id = created_loans[loan_idx % created_loans.len()];
                        if let Ok(loan) = std::panic::catch_unwind(AssertUnwindSafe(|| setup.creditline.get_loan(&loan_id))) {
                            if loan.status == LoanStatus::Active && loan.remaining_balance > 0 {
                                let repay_amount = (loan.remaining_balance * repay_percent as i128) / 100;
                                if repay_amount > 0 {
                                    setup.token_admin_client.mint(&loan.borrower, &repay_amount);
                                    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                                        setup.creditline.repay_loan(&loan.borrower, &loan_id, &repay_amount)
                                    }));
                                }
                            }
                        }
                    }
                }
            }

            // Assert cross-contract accounting invariants after each action step
            assert_invariants(&setup, &provider_addrs, &created_loans);
        }
    }
}

// -------------------------------------------------------------------------
// Storage Boundary & Fuzz Edge-Case Tests
// -------------------------------------------------------------------------

#[test]
fn test_fuzz_boundary_invalid_amounts() {
    let setup = TestEnv::setup();
    let env = &setup.env;
    let provider = Address::generate(env);
    setup.token_admin_client.mint(&provider, &1_000_000);

    // Deposit 0 or negative must panic/fail with InvalidAmount
    let res_zero = std::panic::catch_unwind(AssertUnwindSafe(|| {
        setup.liquidity_pool.deposit(&provider, &0);
    }));
    assert!(res_zero.is_err());

    let res_negative = std::panic::catch_unwind(AssertUnwindSafe(|| {
        setup.liquidity_pool.deposit(&provider, &-500);
    }));
    assert!(res_negative.is_err());

    // Withdraw 0 or negative shares
    setup.liquidity_pool.deposit(&provider, &1_000);
    let res_withdraw_zero = std::panic::catch_unwind(AssertUnwindSafe(|| {
        setup.liquidity_pool.withdraw(&provider, &0);
    }));
    assert!(res_withdraw_zero.is_err());

    // Withdraw more shares than owned
    let res_withdraw_overflow = std::panic::catch_unwind(AssertUnwindSafe(|| {
        setup.liquidity_pool.withdraw(&provider, &999_999_999);
    }));
    assert!(res_withdraw_overflow.is_err());

    // Invariants must still hold after all failed/invalid boundary calls
    assert_invariants(&setup, &[provider], &[]);
}

#[test]
fn test_fuzz_boundary_max_values() {
    let setup = TestEnv::setup();
    let env = &setup.env;
    let provider = Address::generate(env);

    // Attempting i128::MAX deposit without balance or handling safe arithmetic limits
    let res_max = std::panic::catch_unwind(AssertUnwindSafe(|| {
        setup.liquidity_pool.deposit(&provider, &i128::MAX);
    }));
    assert!(res_max.is_err());

    // Verify invariants intact
    assert_invariants(&setup, &[provider], &[]);
}
