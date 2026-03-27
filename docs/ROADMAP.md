# TrustUp Smart Contracts - Development Roadmap

This document provides a comprehensive view of the development status for all smart contract issues across the TrustUp platform.

**Legend:**
- ✅ **Completed** — Fully implemented and tested
- ⚠️ **Incomplete** — Partially implemented or missing key functionality
- ⏳ **Pending** — Not yet started
- 🚧 **In Progress** — Currently being worked on

---

## Phase 1 — Access Control & Governance ✅

**Status:** COMPLETED
**Contract:** [reputation-contract](../contracts/reputation-contract/)

Establishes admin management and updater authorization to secure all contract mutations.

### SC-01: Implement admin management ✅

**Status:** Completed
**Files:**
- [access.rs](../contracts/reputation-contract/src/access.rs)
- [lib.rs:105-125](../contracts/reputation-contract/src/lib.rs#L105-L125)

**Implementation:**
- ✅ Admin initialization (`set_admin`)
- ✅ Admin transfer with authorization check
- ✅ `get_admin()` public accessor
- ✅ Event emission on admin change (`ADMINCHGD`)
- ✅ Strict authorization validation

**Tests:** 26+ tests including:
- Admin succession and transfer
- Permission revocation for old admin
- Admin preservation during state changes

---

### SC-02: Implement updater authorization ✅

**Status:** Completed
**Files:**
- [access.rs](../contracts/reputation-contract/src/access.rs)
- [lib.rs:90-101](../contracts/reputation-contract/src/lib.rs#L90-L101)

**Implementation:**
- ✅ Register updaters (`set_updater`)
- ✅ Revoke updaters
- ✅ Query updater status (`is_updater`)
- ✅ Admin-only mutation with `require_admin`
- ✅ Updater-only function restrictions (`require_updater`)

**Tests:**
- Multiple updater management
- Updater permission revocation
- Unauthorized access prevention

---

### SC-03: Emit access control events ✅

**Status:** Completed
**Files:**
- [events.rs](../contracts/reputation-contract/src/events.rs)

**Implementation:**
- ✅ `ADMINCHGD` — Admin transfer event
- ✅ `UPDCHGD` — Updater status change event
- ✅ Events emitted on all access control mutations

**Tests:**
- Event emission verification for all scenarios
- Event data validation (topics and payloads)

---

## Phase 2 — On-Chain Reputation ✅

**Status:** COMPLETED
**Contract:** [reputation-contract](../contracts/reputation-contract/)

Implements on-chain storage and management of user reputation scores.

### SC-04: Implement reputation storage ✅

**Status:** Completed
**Files:**
- [storage.rs](../contracts/reputation-contract/src/storage.rs)
- [types.rs](../contracts/reputation-contract/src/types.rs)

**Implementation:**
- ✅ On-chain score storage (u32: 0-100)
- ✅ Optimized read/write operations
- ✅ Storage key constants (`SCORES_KEY`)
- ✅ Safe arithmetic with overflow/underflow checks

**Tests:**
- Score persistence across operations
- Storage integrity during admin changes

---

### SC-05: Implement get reputation function ✅

**Status:** Completed
**Files:**
- [lib.rs:27-29](../contracts/reputation-contract/src/lib.rs#L27-L29)

**Implementation:**
- ✅ Public read-only accessor
- ✅ Returns 0 for users without scores (default)
- ✅ Efficient single storage read

**Tests:**
- Default score behavior
- Score retrieval accuracy

---

### SC-06: Implement increase reputation ✅

**Status:** Completed
**Files:**
- [lib.rs:32-51](../contracts/reputation-contract/src/lib.rs#L32-L51)

**Implementation:**
- ✅ Updater-only authorization
- ✅ Overflow protection (max 100)
- ✅ Event emission with reason
- ✅ Zero-amount increases allowed

**Tests:**
- Overflow prevention
- Max score boundary (100)
- Unauthorized access rejection

---

### SC-07: Implement decrease reputation ✅

**Status:** Completed
**Files:**
- [lib.rs:54-69](../contracts/reputation-contract/src/lib.rs#L54-L69)

**Implementation:**
- ✅ Updater-only authorization
- ✅ Underflow protection (min 0)
- ✅ Event emission with reason
- ✅ Zero-amount decreases allowed

**Tests:**
- Underflow prevention
- Min score boundary (0)
- Unauthorized access rejection

---

## Phase 3 — CreditLine Core ✅

**Status:** COMPLETED
**Contract:** [creditline-contract](../contracts/creditline-contract/)

Handles loan creation, repayment, default management, per-user active debt caps, and safe cancellation of unfunded requests.

### SC-08: Implement loan creation ✅

**Status:** Completed
**Files:**
- [lib.rs:52-100](../contracts/creditline-contract/src/lib.rs#L52-L100)
- [types.rs](../contracts/creditline-contract/src/types.rs)

**Implementation:**
- ✅ Loan creation with validation
- ✅ Guarantee validation (minimum 20%)
- ✅ Merchant validation (stubbed for Phase 5)
- ✅ Reputation threshold check (min score 40)
- ✅ Liquidity validation (stubbed for Phase 6)
- ✅ Event emission (`LoanCreated`)
- ✅ Loan counter auto-increment

**Tests:** 15+ tests including:
- Zero/negative amount rejection
- Insufficient guarantee (19%, 10%)
- Exact minimum guarantee edge cases
- Contract initialization

**Known Limitations:**
- ⚠️ Merchant validation bypassed when registry not configured
- ⚠️ Liquidity validation bypassed when pool not configured

---

### SC-09: Implement loan repayment ✅

**Status:** Completed
**Files:**
- [lib.rs](../contracts/creditline-contract/src/lib.rs)
- [types.rs](../contracts/creditline-contract/src/types.rs)
- [tests.rs](../contracts/creditline-contract/src/tests.rs)

**Implementation:**
- ✅ `repay_loan()` supports partial and full repayments
- ✅ Remaining balance and per-component debt tracking (`principal`, `interest`, `service fee`)
- ✅ Full repayment transitions loan to `Paid`
- ✅ Escrowed guarantee is refunded on successful completion
- ✅ Repayments are forwarded through the liquidity-pool interface
- ✅ `LoanRepaid` events emitted

**Tests:**
- Partial repayment scenarios
- Full repayment completion
- Overpayment handling
- Unauthorized repayment attempts
- Repayment on non-active loans

---

### SC-10: Implement loan default ✅

**Status:** Completed
**Files:**
- [lib.rs:222-276](../contracts/creditline-contract/src/lib.rs#L222-L276)

**Implementation:**
- ✅ Mark loans as defaulted
- ✅ Validate loan exists and is active
- ✅ Check overdue status (past final payment date)
- ✅ Guarantee forfeiture logic
- ✅ Event emission (`LoanDefaulted`)
- ✅ Status transition to `Defaulted`

**Tests:**
- Successful default marking
- Premature default rejection (not yet overdue)
- Loan not found scenarios

**Known Limitations:**
- ⚠️ Token transfer to liquidity pool stubbed (Phase 6 dependency)

---

## Phase 4 — CreditLine ↔ Reputation Integration ✅

**Status:** COMPLETED
**Contracts:** creditline-contract, reputation-contract

Bidirectional integration between credit behavior and reputation scores.

### SC-11: Increase reputation on repayment ✅

**Status:** Completed
**Dependencies:** SC-09

**Implementation:**
- ✅ Full repayment calls `increase_score()` on the reputation contract
- ✅ Standard completion bonus applied on successful payoff
- ✅ Early payoff bonus applied when the loan is completed before the first due date
- ✅ Uses `try_invoke_contract` so reputation failures do not corrupt repayment state

**Tests:**
- Score increase path on full repayment
- Early repayment bonus flow
- Integration path stays non-panicking with mock reputation

---

### SC-12: Decrease reputation on default ✅

**Status:** Completed
**Files:**
- [lib.rs:263-280](../contracts/creditline-contract/src/lib.rs#L263-L280)

**Implementation:**
- ✅ Calls `decrease_score()` on reputation contract
- ✅ Calculates penalty amount (20-30 points) based on loan size
- ✅ Uses `try_invoke_contract` for safe error handling
- ✅ Correctly passes `updater`, `user`, and `amount` parameters

**Tests:**
- ✅ Score decrease on default (tested via MockReputation)
- ✅ Verified correct parameter passing
- ✅ Error handling logic verified

---

## Phase 5 — Merchant Registry ✅

**Status:** COMPLETED
**Contract:** [merchant-registry-contract](../contracts/merchant-registry-contract/)

Validates authorized merchants who can receive loan funding.

### SC-13: Implement merchant registration ✅

**Status:** Completed
**Files:**
- [lib.rs](../contracts/merchant-registry-contract/src/lib.rs)
- [types.rs](../contracts/merchant-registry-contract/src/types.rs)
- [tests.rs](../contracts/merchant-registry-contract/src/tests.rs)

**Implementation:**
- ✅ Admin-only merchant registration
- ✅ Merchant info storage with registration date and active flag
- ✅ Merchant count tracking
- ✅ Merchant registration and status events

**Tests:**
- Merchant registration
- Duplicate registration prevention
- Unauthorized registration rejection
- Admin-only access control

---

### SC-14: Implement merchant validation ✅

**Status:** Completed
**Files:**
- [lib.rs](../contracts/creditline-contract/src/lib.rs)
- [lib.rs](../contracts/merchant-registry-contract/src/lib.rs)

**Implementation:**
- ✅ CreditLine checks merchant status through `merchant-registry-contract`
- ✅ Active merchants pass validation
- ✅ Inactive and unregistered merchants are rejected
- ✅ Merchant validation errors are surfaced distinctly

**Tests:**
- Active merchant approval
- Inactive merchant rejection
- Unregistered merchant rejection

---

## Phase 6 — Liquidity Pool ✅

**Status:** COMPLETED
**Contract:** [liquidity-pool-contract](../contracts/liquidity-pool-contract/)

Manages liquidity provider deposits and loan funding.

### SC-15: Implement deposit liquidity ✅

**Status:** Completed

**Implementation:**
- ✅ LP deposit flow with share issuance
- ✅ Initial and proportional share calculation
- ✅ SAC token transfers into the pool
- ✅ `LiquidityDeposited` events

**Tests:**
- First deposit (1:1 share ratio)
- Subsequent deposits (proportional shares)
- Zero deposit rejection

---

### SC-16: Implement withdraw liquidity ✅

**Status:** Completed

**Implementation:**
- ✅ LP withdrawal flow with share burning
- ✅ Available-liquidity protection
- ✅ Proportional withdrawal calculation
- ✅ `LiquidityWithdrawn` events

**Tests:**
- Full withdrawal
- Partial withdrawal
- Insufficient liquidity rejection
- Share burning verification

---

### SC-17: Implement interest distribution ✅

**Status:** Completed

**Implementation:**
- ✅ `receive_repayment()` accepts principal and interest
- ✅ `distribute_interest()` routes repayment yield across LP / protocol / merchant buckets
- ✅ Share value appreciates as LP-owned interest stays in the pool
- ✅ Interest-distribution events emitted

**Tests:**
- Interest calculation accuracy
- Share value appreciation
- Fee split verification
- Multiple LP proportional distribution

---

## Phase 7 — Contract Testing ⚠️

**Status:** PARTIALLY COMPLETED

Comprehensive test coverage for all contracts.

### SC-18: Unit tests for Reputation Contract ✅

**Status:** Completed
**Files:**
- [tests.rs](../contracts/reputation-contract/src/tests.rs)

**Test Coverage:** 26+ tests
- ✅ Admin management (6 tests)
- ✅ Updater authorization (5 tests)
- ✅ Score mutations (increase, decrease, set) (8 tests)
- ✅ Overflow/underflow protection (6 tests)
- ✅ Event emission (6 tests)
- ✅ Edge cases (zero amounts, boundary values)

**Coverage Assessment:** Excellent — All core functionality tested

---

### SC-19: Unit tests for CreditLine Contract ⚠️

**Status:** INCOMPLETE
**Files:**
- [tests.rs](../contracts/creditline-contract/src/tests.rs)

**Test Coverage:** 15+ tests
- ✅ Initialization and admin management
- ✅ Loan creation validations (amounts, guarantees)
- ✅ Mark defaulted functionality
- ✅ Contract address updates
- ✅ CreditLine repayment tests cover partial/full repayment and state transitions
- ✅ Reputation integration paths exist for repayment and default
- ✅ Merchant validation tests cover active, inactive, and unregistered merchants
- ❌ **MISSING:** Liquidity pool integration tests (Phase 6)

**Required Tests:**
- Repayment scenarios (partial, full, overpayment)
- Reputation score updates on payment/default
- End-to-end loan lifecycle (create → repay → complete)
- Integration tests with all external contracts

---

### SC-20: Unit tests for Liquidity Pool ⚠️

**Status:** IN PROGRESS
**Reason:** Contract exists and has substantial coverage, but roadmap tracking is still conservative until ignored cross-contract cases are replaced with end-to-end assertions.

**Required Tests:**
- Deposit scenarios (first LP, subsequent LPs)
- Share calculation accuracy
- Withdrawal scenarios (partial, full)
- Interest distribution
- Low liquidity edge cases
- Share value appreciation
- Multiple LP interactions

---

## Summary Dashboard

### Overall Progress: 17/20 Issues Completed (85%)

| Phase | Status | Completed | Total | Progress |
|-------|--------|-----------|-------|----------|
| Phase 1: Access Control | ✅ Complete | 3/3 | 3 | 100% |
| Phase 2: Reputation | ✅ Complete | 4/4 | 4 | 100% |
| Phase 3: CreditLine Core | ✅ Complete | 3/3 | 3 | 100% |
| Phase 4: Integration | ✅ Complete | 2/2 | 2 | 100% |
| Phase 5: Merchant Registry | ✅ Complete | 2/2 | 2 | 100% |
| Phase 6: Liquidity Pool | ✅ Complete | 3/3 | 3 | 100% |
| Phase 7: Testing | ⚠️ Partial | 1/3 | 3 | 33% |

### By Status

- ✅ **Completed:** 17 issues
- ⚠️ **Incomplete/Partial:** 1 issue
- ❌ **Not Started:** 2 issues

---

## Critical Blockers

### 1. SC-19 / SC-20: Broader integration and pool test depth ⚠️

**Impact:** MEDIUM
**Current Issue:** Core contracts are implemented, but cross-contract verification still relies partly on mocks or ignored cases.
**Effort:** Medium

---

## Next Steps (Recommended Order)

1. **Immediate**
   - [ ] Replace mocked pool assertions in CreditLine tests with concrete integration coverage
   - [ ] Expand repayment/default accounting tests across contract boundaries

2. **Short Term**
   - [ ] Complete SC-20 liquidity pool test expansion
   - [ ] Add more end-to-end multi-contract flows without mocked externals

3. **Polish**
   - [ ] Integration testing across all contracts
   - [ ] Security audit preparation
   - [ ] Gas optimization
   - [ ] Documentation updates

---

## Unplanned Contracts

### adapter-trustless-contract

**Status:** Empty (`.gitkeep` only)
**Purpose:** Unknown — not in original roadmap
**Action Required:** Clarify scope or remove

---

## Notes

- All completed phases (1-2) have excellent test coverage
- CreditLine now supports priced loans, reputation-linked limits, and pending-loan cancellation
- No integration tests exist yet between contracts
- Security considerations documented in [ERROR_CODES.md](ERROR_CODES.md)

---

**Last Updated:** 2026-03-27
**Document Owner:** TrustUp Development Team
**Related Docs:** [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md) | [ARCHITECTURE.md](ARCHITECTURE.md) | [CONTRIBUTING.md](CONTRIBUTING.md)
