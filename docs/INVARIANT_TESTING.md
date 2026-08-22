# Property-Based Invariant Testing Guide

This document describes the property-based testing and cross-contract invariant validation suite built for TrustUp Smart Contracts using `proptest`.

---

## 🎯 Overview

Unlike unit tests that check point-in-time state or deterministic function inputs, **property-based invariant tests** generate randomized sequences of operations (deposits, withdrawals, loan creations, partial/full repayments, and edge-case amounts) to verify that high-level system invariants are never violated under any valid operation sequence.

---

## 🛡️ Core System Invariants

The test suite in [`tests/integration/src/test_invariants.rs`](file:///c:/Trabajos%20Progra/GRANDFOX/TrustUp-Contracts/tests/integration/src/test_invariants.rs) continuously asserts four fundamental accounting invariants:

1. **Share Supply Integrity (`liquidity-pool-contract`)**
   $$\text{total\_shares} = \sum_{\text{provider}} \text{provider\_shares}$$
   The sum of shares held across all active liquidity providers must always equal `total_shares` in the pool storage.

2. **Liquidity Accounting (`liquidity-pool-contract`)**
   $$\text{available\_liquidity} + \text{locked\_liquidity} = \text{total\_liquidity}$$
   The pool's reported `available_liquidity` plus active `locked_liquidity` must strictly equal `total_liquidity`.

3. **Cross-Contract Locked Balance Reconciliation**
   $$\text{locked\_liquidity}_{\text{LP}} = \sum_{\text{active loans}} \text{principal\_outstanding}_{\text{pool}}$$
   The sum of active loan principal funded by the pool must equal the pool's internal `locked_liquidity`.

4. **Underlying Token Balance Consistency**
   $$\text{Balance}_{\text{token}}(\text{LP Address}) = \text{available\_liquidity}$$
   The real Stellar SEP-41 token balance held by the liquidity pool contract address must equal its internal `available_liquidity` accounting.

---

## 🎲 Scenario & Operation Generator

The suite utilizes a randomized operation generator producing sequences of `Action` commands:
- **`Action::Deposit`**: Multi-provider deposits with randomized amounts ranging from $1$ unit up to $1,000,000,000$ stroops.
- **`Action::Withdraw`**: Provider share redemptions calculated in basis points ($100$ to $10,000$ bps).
- **`Action::CreateLoan`**: BNPL loan creations with variable amounts, merchant destinations, and guarantee percentages ($20\%$ to $50\%$).
- **`Action::RepayLoan`**: Partial and full loan repayments ($10\%$ to $100\%$ of remaining loan balances).

---

## ⚡ Storage Boundary & Fuzz Edge Cases

In addition to property sequence testing, the suite includes fuzz tests targeting:
- Invalid deposit amounts ($0$ or negative numbers).
- Excessive share withdrawal attempts beyond owned shares or available liquidity.
- Overflow boundary values (`i128::MAX`).

All boundary tests assert that invalid calls fail safely with appropriate contract error codes (e.g. `LiquidityPoolError::InvalidAmount`) while preserving all core invariants intact.

---

## 🚀 Running Property Tests

To run the property invariant suite locally:

```bash
# Run integration & property invariant tests
cargo test -p integration-tests

# Run specifically the property invariant tests
cargo test -p integration-tests test_property_invariants_random_sequences
```

---

## ➕ Adding New Invariants Going Forward

When adding new contract features or additional financial mechanisms:

1. **Define the Invariant Function**:
   Add a new assertion block inside `assert_invariants()` in [`tests/integration/src/test_invariants.rs`](file:///c:/Trabajos%20Progra/GRANDFOX/TrustUp-Contracts/tests/integration/src/test_invariants.rs).

2. **Extend the `Action` Enum**:
   If the new feature introduces a new state-changing call (e.g. default liquidations, fee distribution), add an `Action` variant and strategy in `prop_action_strategy()`.

3. **Update Action Execution**:
   Add a match arm inside `test_property_invariants_random_sequences` to execute the action during property test iterations.

4. **Verify Clean Execution**:
   Run `cargo test -p integration-tests` to confirm no property counterexamples are found.
