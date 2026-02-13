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

## Phase 3 — CreditLine Core ⚠️

**Status:** PARTIALLY COMPLETED
**Contract:** [creditline-contract](../contracts/creditline-contract/)

Handles loan creation, repayment, and default management.

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

### SC-09: Implement loan repayment ❌

**Status:** NOT IMPLEMENTED
**Files:** None

**Missing Functionality:**
- ❌ `repay_loan()` function does not exist
- ❌ Partial payment support
- ❌ Full repayment logic
- ❌ Remaining balance updates
- ❌ Payment event emission (`LoanRepaid`)
- ❌ Loan status transition to `Repaid`

**Required Implementation:**
```rust
pub fn repay_loan(
    env: Env,
    borrower: Address,
    loan_id: u64,
    amount: i128
) -> i128 // returns remaining balance
```

**Tests Needed:**
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
- ⚠️ Calls reputation contract's `slash` method which doesn't exist (should call `decrease_score`)
- ⚠️ Token transfer to liquidity pool stubbed (Phase 6 dependency)

---

## Phase 4 — CreditLine ↔ Reputation Integration ⚠️

**Status:** INCOMPLETE
**Contracts:** creditline-contract, reputation-contract

Bidirectional integration between credit behavior and reputation scores.

### SC-11: Increase reputation on repayment ❌

**Status:** NOT IMPLEMENTED
**Dependencies:** SC-09 (loan repayment)

**Missing Functionality:**
- ❌ No call to reputation contract on successful repayment
- ❌ Score increase logic not implemented
- ❌ Repayment callback missing

**Required Work:**
- Implement `repay_loan()` function
- Add reputation contract invocation:
  ```rust
  env.invoke_contract::<()>(
      &reputation_contract,
      &symbol_short!("increase_score"),
      (updater, borrower, amount).into_val(&env)
  );
  ```

**Tests Needed:**
- Score increase on full repayment
- Score increase on on-time payment
- Early payment bonus logic
- Integration with reputation contract

---

### SC-12: Decrease reputation on default ⚠️

**Status:** INCOMPLETE
**Files:**
- [lib.rs:263-272](../contracts/creditline-contract/src/lib.rs#L263-L272)

**Current Implementation:**
- ⚠️ Calls `reputation_contract.slash()` which **does not exist**
- ⚠️ Should call `decrease_score()` instead
- ⚠️ No explicit reason/amount passed

**Required Fixes:**
```rust
// Current (INCORRECT):
env.invoke_contract::<()>(
    &reputation_contract,
    &symbol_short!("slash"),
    (loan.borrower,).into_val(&env)
);

// Should be:
env.invoke_contract::<()>(
    &reputation_contract,
    &symbol_short!("decrease_score"),
    (creditline_updater, loan.borrower, penalty_amount).into_val(&env)
);
```

**Tests Needed:**
- Score decrease on default
- Correct penalty amount calculation
- Event verification for score change

---

## Phase 5 — Merchant Registry ⏳

**Status:** NOT STARTED
**Contract:** merchant-registry-contract (empty)

Validates authorized merchants who can receive loan funding.

### SC-13: Implement merchant registration ❌

**Status:** NOT IMPLEMENTED
**Files:** `contracts/merchant-registry-contract/.gitkeep` (empty directory)

**Required Implementation:**
- ❌ `register_merchant(admin, merchant, metadata)`
- ❌ Merchant storage structure
- ❌ Admin-only registration
- ❌ Event emission (`MerchantRegistered`)
- ❌ Metadata fields (name, category, etc.)

**Tests Needed:**
- Merchant registration
- Duplicate registration prevention
- Unauthorized registration rejection
- Admin-only access control

---

### SC-14: Implement merchant validation ⚠️

**Status:** STUBBED (not functional)
**Files:**
- [lib.rs:162-177](../contracts/creditline-contract/src/lib.rs#L162-L177)

**Current Implementation:**
- ⚠️ Validation bypassed when registry not configured
- ⚠️ Always assumes merchant is valid
- ⚠️ TODO comment indicates placeholder

**Required Implementation:**
```rust
pub fn is_active_merchant(env: Env, merchant: Address) -> bool

// In CreditLine:
let is_valid: bool = env.invoke_contract(
    &merchant_registry,
    &symbol_short!("is_active"),
    (merchant,).into_val(&env)
);
```

**Tests Needed:**
- Active merchant approval
- Inactive merchant rejection
- Unregistered merchant rejection

---

## Phase 6 — Liquidity Pool ⏳

**Status:** NOT STARTED
**Contract:** None (does not exist)

Manages liquidity provider deposits and loan funding.

### SC-15: Implement deposit liquidity ❌

**Status:** NOT IMPLEMENTED

**Required Implementation:**
- ❌ Contract creation
- ❌ `deposit(provider, amount)` function
- ❌ Share calculation and issuance
- ❌ Token transfer handling (SAC tokens)
- ❌ Event emission (`LiquidityDeposited`)

**Tests Needed:**
- First deposit (1:1 share ratio)
- Subsequent deposits (proportional shares)
- Share value calculation accuracy
- Zero deposit rejection

---

### SC-16: Implement withdraw liquidity ❌

**Status:** NOT IMPLEMENTED

**Required Implementation:**
- ❌ `withdraw(provider, shares)` function
- ❌ Share burning logic
- ❌ Available liquidity checks
- ❌ Withdrawal amount calculation
- ❌ Event emission (`LiquidityWithdrawn`)

**Tests Needed:**
- Full withdrawal
- Partial withdrawal
- Insufficient liquidity rejection
- Share burning verification

---

### SC-17: Implement interest distribution ❌

**Status:** NOT IMPLEMENTED

**Required Implementation:**
- ❌ `distribute_interest()` function
- ❌ Interest accumulation from repayments
- ❌ Share value increase logic
- ❌ Fee distribution (85% LP, 10% protocol, 5% merchant)
- ❌ Event emission (`InterestDistributed`)

**Tests Needed:**
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
- ❌ **MISSING:** Loan repayment tests (SC-09 not implemented)
- ❌ **MISSING:** Reputation integration tests
- ❌ **MISSING:** Merchant validation tests (Phase 5)
- ❌ **MISSING:** Liquidity pool integration tests (Phase 6)

**Required Tests:**
- Repayment scenarios (partial, full, overpayment)
- Reputation score updates on payment/default
- End-to-end loan lifecycle (create → repay → complete)
- Integration tests with all external contracts

---

### SC-20: Unit tests for Liquidity Pool ❌

**Status:** NOT STARTED
**Reason:** Contract does not exist

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

### Overall Progress: 11/20 Issues Completed (55%)

| Phase | Status | Completed | Total | Progress |
|-------|--------|-----------|-------|----------|
| Phase 1: Access Control | ✅ Complete | 3/3 | 3 | 100% |
| Phase 2: Reputation | ✅ Complete | 4/4 | 4 | 100% |
| Phase 3: CreditLine Core | ⚠️ Partial | 2/3 | 3 | 67% |
| Phase 4: Integration | ⚠️ Partial | 0/2 | 2 | 0% |
| Phase 5: Merchant Registry | ⏳ Pending | 0/2 | 2 | 0% |
| Phase 6: Liquidity Pool | ⏳ Pending | 0/3 | 3 | 0% |
| Phase 7: Testing | ⚠️ Partial | 1/3 | 3 | 33% |

### By Status

- ✅ **Completed:** 11 issues
- ⚠️ **Incomplete/Partial:** 4 issues
- ❌ **Not Started:** 5 issues

---

## Critical Blockers

### 1. SC-09: Implement loan repayment ❌

**Impact:** HIGH
**Blocks:**
- SC-11 (reputation increase on repayment)
- SC-19 (CreditLine tests incomplete)
- Core BNPL functionality unusable

**Effort:** Medium (2-3 days)

---

### 2. SC-12: Fix reputation decrease on default ⚠️

**Impact:** MEDIUM
**Current Issue:** Calls non-existent `slash()` method instead of `decrease_score()`
**Effort:** Low (1-2 hours)

---

### 3. SC-13, SC-14: Merchant Registry ❌

**Impact:** MEDIUM
**Current Workaround:** Validation bypassed in CreditLine
**Security Risk:** Any address can be treated as valid merchant
**Effort:** Medium (2-3 days)

---

### 4. SC-15, SC-16, SC-17: Liquidity Pool ❌

**Impact:** HIGH
**Current Issue:** No funding source for loans
**Blocks:** End-to-end loan flow testing
**Effort:** High (1-2 weeks)

---

## Next Steps (Recommended Order)

1. **Immediate (Week 1)**
   - [ ] Fix SC-12: Update `mark_defaulted` to call `decrease_score` properly
   - [ ] Implement SC-09: `repay_loan()` function in CreditLine
   - [ ] Implement SC-11: Add reputation increase on repayment

2. **Short Term (Week 2-3)**
   - [ ] Implement SC-13: Merchant Registry contract
   - [ ] Implement SC-14: Merchant validation integration
   - [ ] Update SC-19: Add comprehensive CreditLine tests

3. **Medium Term (Month 1-2)**
   - [ ] Implement SC-15: Liquidity Pool deposit
   - [ ] Implement SC-16: Liquidity Pool withdrawal
   - [ ] Implement SC-17: Interest distribution
   - [ ] Implement SC-20: Liquidity Pool tests

4. **Polish (Month 2+)**
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
- CreditLine contract is well-structured but missing key repayment logic
- No integration tests exist yet between contracts
- Security considerations documented in [ERROR_CODES.md](ERROR_CODES.md)

---

**Last Updated:** 2026-02-13
**Document Owner:** TrustUp Development Team
**Related Docs:** [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md) | [ARCHITECTURE.md](ARCHITECTURE.md) | [CONTRIBUTING.md](CONTRIBUTING.md)
