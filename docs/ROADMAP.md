# Development Roadmap

## Project Status

**Current Phase**: Phase 2 - Reputation On-Chain ✅ (Completed)
**Next Phase**: Phase 3 - CreditLine Core
**Overall Progress**: 8/20 issues completed (40%)

## Phase Overview

| Phase | Name | Status | Completed | Total | Progress |
|-------|------|--------|-----------|-------|----------|
| 1 | Access Control & Governance | ✅ Complete | 3/3 | 3 | 100% |
| 2 | Reputation On-Chain | ✅ Complete | 4/4 | 4 | 100% |
| 3 | CreditLine Core | ⏳ Pending | 0/3 | 3 | 0% |
| 4 | CreditLine ↔ Reputation | ⏳ Pending | 0/2 | 2 | 0% |
| 5 | Merchant Registry | ⏳ Pending | 0/2 | 2 | 0% |
| 6 | Liquidity Pool | ⏳ Pending | 0/3 | 3 | 0% |
| 7 | Contract Tests | 🚧 In Progress | 1/3 | 3 | 33% |

**Legend**:
- ✅ Completed
- 🚧 In Progress
- ⏳ Pending (not started)

---

## Phase 1: Access Control & Governance

**Goal**: Establish foundation for secure contract administration and role-based access control.

**Dependencies**: None (foundational phase)

**Status**: ✅ **COMPLETED**

### Issues

#### ✅ SC-01: Implement Admin Management
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/lib.rs:77-92](../contracts/reputation-contract/src/lib.rs), [contracts/reputation-contract/src/access.rs:4-18](../contracts/reputation-contract/src/access.rs)

**Description**:
Define initial admin, enable admin transfer, and implement strict validation to prevent unauthorized modifications.

**Requirements**:
- [x] Initialize admin on first setup (no auth required for first call)
- [x] Implement `set_admin(new_admin: Address)` function
- [x] Implement `get_admin() -> Address` function
- [x] Require admin authorization for admin transfer
- [x] Emit `ADMINCHGD` event when admin changes

**Implementation Details**:
```rust
// Function signature
pub fn set_admin(env: Env, new_admin: Address) -> Result<(), ReputationError>

// Access control
- First call: No authorization required (admin initialization)
- Subsequent calls: Requires current admin authorization
```

**Events Emitted**:
```rust
env.events().publish(
    (symbol_short!("ADMINCHGD"),),
    (old_admin, new_admin)
);
```

**Related Files**:
- `contracts/reputation-contract/src/lib.rs` - Main admin functions
- `contracts/reputation-contract/src/access.rs` - Admin authorization checks
- `contracts/reputation-contract/src/storage.rs` - Admin storage operations
- `contracts/reputation-contract/src/events.rs` - Admin change events

---

#### ✅ SC-02: Implement Updater Authorization
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/lib.rs:104-121](../contracts/reputation-contract/src/lib.rs), [contracts/reputation-contract/src/access.rs:20-32](../contracts/reputation-contract/src/access.rs)

**Description**:
Register and revoke authorized updaters. Restrict sensitive score modification functions to these roles.

**Requirements**:
- [x] Implement `set_updater(admin: Address, updater: Address, allowed: bool)` function
- [x] Implement `is_updater(addr: Address) -> bool` function
- [x] Only admin can register/revoke updaters
- [x] Store updater status in contract storage
- [x] Emit `UPDCHGD` event when updater status changes

**Implementation Details**:
```rust
// Function signature
pub fn set_updater(env: Env, admin: Address, updater: Address, allowed: bool)

// Access control
- Requires admin authorization (admin.require_auth())
- Validates caller is stored admin
```

**Events Emitted**:
```rust
env.events().publish(
    (symbol_short!("UPDCHGD"), updater),
    allowed
);
```

**Usage Example**:
```rust
// Admin grants updater permission to CreditLine contract
reputation_contract.set_updater(&admin, &creditline_address, &true);

// Later, admin revokes permission
reputation_contract.set_updater(&admin, &creditline_address, &false);
```

**Related Files**:
- `contracts/reputation-contract/src/lib.rs` - Updater management functions
- `contracts/reputation-contract/src/access.rs` - Updater authorization checks
- `contracts/reputation-contract/src/storage.rs` - Updater storage operations
- `contracts/reputation-contract/src/events.rs` - Updater change events

---

#### ✅ SC-03: Emit Access Control Events
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/events.rs](../contracts/reputation-contract/src/events.rs)

**Description**:
Emit blockchain events when admin or updaters change to enable observability and off-chain indexing.

**Requirements**:
- [x] Define event topics and data structures
- [x] Emit `ADMINCHGD` event on admin transfer
- [x] Emit `UPDCHGD` event on updater status change
- [x] Include relevant addresses in event topics for indexing
- [x] Include old/new values in event data

**Implemented Events**:

1. **Admin Changed Event**:
```rust
Topic: (symbol_short!("ADMINCHGD"),)
Data: (old_admin: Address, new_admin: Address)
```

2. **Updater Changed Event**:
```rust
Topic: (symbol_short!("UPDCHGD"), updater: Address)
Data: allowed: bool
```

**Related Files**:
- `contracts/reputation-contract/src/events.rs` - Event emission helpers
- `contracts/reputation-contract/src/lib.rs` - Event calls in admin/updater functions

---

## Phase 2: Reputation On-Chain

**Goal**: Implement on-chain reputation scoring system with secure storage and update mechanisms.

**Dependencies**: Phase 1 (requires admin and updater roles)

**Status**: ✅ **COMPLETED**

### Issues

#### ✅ SC-04: Implement Reputation Storage
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/storage.rs](../contracts/reputation-contract/src/storage.rs)

**Description**:
Define on-chain data structure for reputation scores with minimal metadata. Optimize for efficient read/write operations.

**Requirements**:
- [x] Define storage key structure for user scores
- [x] Implement `get_score(user: Address) -> u32` storage function
- [x] Implement `set_score(user: Address, score: u32)` storage function
- [x] Use Map-based storage for user scores
- [x] Return default score (0) for users without score history

**Implementation Details**:
```rust
// Storage key enum
pub enum ReputationDataKey {
    Admin,
    Updater(Address),
    Score(Address),
}

// Storage operations use Soroban Map
env.storage().instance().get(&key).unwrap_or(DEFAULT_SCORE)
env.storage().instance().set(&key, &score)
```

**Storage Schema**:
| Key | Value Type | Description |
|-----|-----------|-------------|
| `Admin` | `Address` | Current admin address |
| `Updater(Address)` | `bool` | Whether address is authorized updater |
| `Score(Address)` | `u32` | User's reputation score (0-100) |

**Related Files**:
- `contracts/reputation-contract/src/storage.rs` - Storage implementation
- `contracts/reputation-contract/src/types.rs` - DEFAULT_SCORE constant (0)

---

#### ✅ SC-05: Implement Get Reputation Function
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/lib.rs:29-37](../contracts/reputation-contract/src/lib.rs)

**Description**:
Expose public read function for querying user reputation scores. Ensure secure and efficient access.

**Requirements**:
- [x] Implement `get_score(user: Address) -> u32` public function
- [x] No authorization required (public read)
- [x] Return stored score or default (0) if not found
- [x] Optimize for gas efficiency

**Implementation Details**:
```rust
pub fn get_score(env: Env, user: Address) -> u32 {
    storage::get_score(&env, &user)
}
```

**Usage Example**:
```rust
// Query user's reputation score
let score = reputation_contract.get_score(&user_address);

// Use score to determine credit terms
if score >= 75 {
    // Offer premium credit terms
} else if score >= 50 {
    // Offer standard credit terms
} else {
    // Deny credit or offer limited terms
}
```

**Related Files**:
- `contracts/reputation-contract/src/lib.rs` - Public interface
- `contracts/reputation-contract/src/storage.rs` - Storage read operation

---

#### ✅ SC-06: Implement Increase Reputation
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/lib.rs:39-58](../contracts/reputation-contract/src/lib.rs)

**Description**:
Allow authorized updaters to increase user scores with validation. Prevent overflow and out-of-bounds errors.

**Requirements**:
- [x] Implement `increase_score(updater: Address, user: Address, amount: u32)` function
- [x] Require updater authorization
- [x] Use checked arithmetic to prevent overflow
- [x] Validate result is within bounds (0-100)
- [x] Emit `SCORECHGD` event with reason

**Implementation Details**:
```rust
pub fn increase_score(env: Env, updater: Address, user: Address, amount: u32) {
    access::require_updater(&env, &updater);

    let old_score = storage::get_score(&env, &user);
    let new_score = old_score
        .checked_add(amount)
        .ok_or_else(|| ReputationError::Overflow)
        .unwrap();

    if new_score > types::MAX_SCORE {
        panic_with_error!(&env, ReputationError::OutOfBounds);
    }

    storage::set_score(&env, &user, new_score);
    events::emit_score_changed(&env, &user, old_score, new_score, symbol_short!("increase"));
}
```

**Validation Rules**:
- Amount must be non-negative
- Result must not overflow u32
- Result must not exceed MAX_SCORE (100)

**Events Emitted**:
```rust
Topic: (symbol_short!("SCORECHGD"), user)
Data: (old_score: u32, new_score: u32, reason: Symbol("increase"))
```

**Related Files**:
- `contracts/reputation-contract/src/lib.rs` - Increase score function
- `contracts/reputation-contract/src/access.rs` - Updater authorization
- `contracts/reputation-contract/src/events.rs` - Score change event

---

#### ✅ SC-07: Implement Decrease Reputation
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/lib.rs:60-75](../contracts/reputation-contract/src/lib.rs)

**Description**:
Allow authorized updaters to decrease user scores with explicit cause. Prevent underflow errors.

**Requirements**:
- [x] Implement `decrease_score(updater: Address, user: Address, amount: u32)` function
- [x] Require updater authorization
- [x] Use checked arithmetic to prevent underflow
- [x] Validate result is within bounds (0-100)
- [x] Emit `SCORECHGD` event with reason

**Implementation Details**:
```rust
pub fn decrease_score(env: Env, updater: Address, user: Address, amount: u32) {
    access::require_updater(&env, &updater);

    let old_score = storage::get_score(&env, &user);
    let new_score = old_score
        .checked_sub(amount)
        .ok_or_else(|| ReputationError::Underflow)
        .unwrap();

    storage::set_score(&env, &user, new_score);
    events::emit_score_changed(&env, &user, old_score, new_score, symbol_short!("decrease"));
}
```

**Validation Rules**:
- Amount must be non-negative
- Result must not underflow (go below 0)

**Events Emitted**:
```rust
Topic: (symbol_short!("SCORECHGD"), user)
Data: (old_score: u32, new_score: u32, reason: Symbol("decrease"))
```

**Use Cases**:
- Late payment: -5 points
- Missed payment: -10 points
- Loan default: -30 points

**Related Files**:
- `contracts/reputation-contract/src/lib.rs` - Decrease score function
- `contracts/reputation-contract/src/access.rs` - Updater authorization
- `contracts/reputation-contract/src/events.rs` - Score change event

---

## Phase 3: CreditLine Core

**Goal**: Implement core loan management functionality including creation, repayment, and default tracking.

**Dependencies**: Phase 1 (requires access control), Phase 2 (requires reputation queries)

**Status**: ⏳ **PENDING**

### Issues

#### ⏳ SC-08: Implement Loan Creation
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Create loans with amount, guarantee deposit, repayment dates, and initial state. Validate all inputs and emit loan creation event.

**Requirements**:
- [ ] Define loan data structure (amount, guarantee, dates, merchant, status)
- [ ] Implement `create_loan(...)` function with all required parameters
- [ ] Validate user has sufficient guarantee deposit
- [ ] Validate merchant is registered (query Merchant Registry)
- [ ] Query user reputation to determine credit terms
- [ ] Request funds from Liquidity Pool
- [ ] Store loan with unique ID
- [ ] Emit `LoanCreated` event

**Proposed Function Signature**:
```rust
pub fn create_loan(
    env: Env,
    user: Address,
    merchant: Address,
    total_amount: i128,
    guarantee_amount: i128,
    repayment_schedule: Vec<RepaymentInstallment>,
) -> u64  // Returns loan_id
```

**Validation Rules**:
- `guarantee_amount` >= 20% of `total_amount`
- Merchant must be active in Merchant Registry
- User must meet minimum reputation threshold
- Liquidity Pool must have sufficient funds

**State Transitions**:
```
[No Loan] --create_loan()--> [Active Loan]
```

**Events to Emit**:
```rust
Topic: (symbol_short!("LOANCRTD"), user, merchant)
Data: (loan_id, total_amount, guarantee_amount, repayment_schedule)
```

**Related Contracts**:
- Queries: Reputation Contract, Merchant Registry Contract
- Transfers: Liquidity Pool Contract

---

#### ⏳ SC-09: Implement Loan Repayment
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Process loan repayment installments (partial and full payments). Update loan state and emit payment events.

**Requirements**:
- [ ] Implement `repay_loan(loan_id: u64, amount: i128)` function
- [ ] Validate loan exists and is active
- [ ] Validate caller is loan borrower
- [ ] Accept partial and full repayments
- [ ] Update remaining balance
- [ ] Transfer funds to Liquidity Pool
- [ ] Mark loan as paid if fully repaid
- [ ] Emit `LoanRepaid` event

**Proposed Function Signature**:
```rust
pub fn repay_loan(
    env: Env,
    borrower: Address,
    loan_id: u64,
    amount: i128,
) -> Result<LoanStatus, CreditLineError>
```

**Validation Rules**:
- Loan must exist
- Loan must be in Active status
- Caller must be the borrower
- Amount must be positive
- Amount must not exceed remaining balance

**State Transitions**:
```
[Active Loan] --repay_loan(partial)--> [Active Loan] (updated balance)
[Active Loan] --repay_loan(full)--> [Paid Loan]
```

**Events to Emit**:
```rust
Topic: (symbol_short!("LOANPAID"), borrower, loan_id)
Data: (amount, remaining_balance, timestamp)
```

**Related Contracts**:
- Transfers: Liquidity Pool Contract (receive repayment)

---

#### ⏳ SC-10: Implement Loan Default
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Mark overdue loans as defaulted. Handle guarantee forfeiture and update loan status.

**Requirements**:
- [ ] Implement `mark_defaulted(loan_id: u64)` function
- [ ] Validate loan is overdue (past final payment date)
- [ ] Transfer guarantee to Liquidity Pool
- [ ] Update loan status to Defaulted
- [ ] Emit `LoanDefaulted` event
- [ ] Trigger reputation decrease (Phase 4 integration)

**Proposed Function Signature**:
```rust
pub fn mark_defaulted(
    env: Env,
    loan_id: u64,
) -> Result<(), CreditLineError>
```

**Validation Rules**:
- Loan must exist
- Loan must be in Active status
- Current timestamp must be past final payment date
- Can be called by anyone (permissionless enforcement)

**State Transitions**:
```
[Active Loan] --mark_defaulted()--> [Defaulted Loan]
```

**Events to Emit**:
```rust
Topic: (symbol_short!("LOANDFLT"), borrower, loan_id)
Data: (total_amount, unpaid_balance, guarantee_forfeited, timestamp)
```

**Related Contracts**:
- Transfers: Liquidity Pool Contract (receive forfeited guarantee)
- Updates: Reputation Contract (decrease score - Phase 4)

---

## Phase 4: CreditLine ↔ Reputation

**Goal**: Integrate CreditLine contract with Reputation contract to automatically adjust scores based on payment behavior.

**Dependencies**: Phase 2 (Reputation), Phase 3 (CreditLine)

**Status**: ⏳ **PENDING**

### Issues

#### ⏳ SC-11: Increase Reputation on Repayment
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Automatically increase user reputation score when loan payments are made successfully.

**Requirements**:
- [ ] CreditLine contract registered as updater in Reputation contract
- [ ] Call `increase_score()` on successful repayment
- [ ] Calculate reputation increase based on payment behavior:
  - On-time payment: +2 to +5 points
  - Early payment: +5 to +10 points
  - Full loan completion: +10 points
- [ ] Only increase reputation if payment is on-time or early
- [ ] Handle reputation contract errors gracefully

**Proposed Implementation**:
```rust
// In CreditLine contract
pub fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) {
    // ... existing repayment logic ...

    // Calculate reputation increase
    let reputation_increase = calculate_reputation_boost(&env, &loan, amount);

    // Update reputation
    reputation_contract.increase_score(&env.current_contract_address(), &borrower, &reputation_increase);
}
```

**Reputation Increase Logic**:
```rust
fn calculate_reputation_boost(loan: &Loan, payment_amount: i128) -> u32 {
    if is_early_payment(loan) {
        10  // Bonus for early payment
    } else if is_on_time_payment(loan) {
        5   // Standard increase for on-time
    } else if is_full_payment(loan, payment_amount) {
        15  // Bonus for paying off entire loan
    } else {
        2   // Minimum increase for any payment
    }
}
```

**Edge Cases**:
- Partial payments should give proportional increases
- Late payments should NOT increase reputation
- Multiple payments in same day should only count once

**Related Contracts**:
- Updates: Reputation Contract (`increase_score`)

---

#### ⏳ SC-12: Decrease Reputation on Default
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Automatically decrease user reputation score when loans enter default status.

**Requirements**:
- [ ] CreditLine contract registered as updater in Reputation contract
- [ ] Call `decrease_score()` when loan marked as defaulted
- [ ] Calculate reputation decrease based on default severity:
  - Single late payment: -5 points
  - Missed payment: -10 points
  - Full default: -30 points
- [ ] Record explicit reason for reputation decrease
- [ ] Handle reputation contract errors gracefully

**Proposed Implementation**:
```rust
// In CreditLine contract
pub fn mark_defaulted(env: Env, loan_id: u64) {
    // ... existing default logic ...

    let loan = get_loan(&env, loan_id)?;

    // Calculate reputation penalty based on default severity
    let reputation_penalty = calculate_default_penalty(&loan);

    // Update reputation
    reputation_contract.decrease_score(
        &env.current_contract_address(),
        &loan.borrower,
        &reputation_penalty
    );
}
```

**Reputation Decrease Logic**:
```rust
fn calculate_default_penalty(loan: &Loan) -> u32 {
    let unpaid_ratio = loan.remaining_balance / loan.total_amount;

    if unpaid_ratio >= 0.9 {
        30  // Defaulted early, almost no payments made
    } else if unpaid_ratio >= 0.5 {
        20  // Defaulted midway
    } else {
        10  // Defaulted near end
    }
}
```

**Reputation Reason Codes**:
- `"late"` - Late payment
- `"missed"` - Missed payment deadline
- `"default"` - Full loan default

**Related Contracts**:
- Updates: Reputation Contract (`decrease_score`)

---

## Phase 5: Merchant Registry

**Goal**: Implement merchant whitelist and validation system.

**Dependencies**: Phase 1 (requires admin for merchant management)

**Status**: ⏳ **PENDING**

### Issues

#### ⏳ SC-13: Implement Merchant Registration
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Allow admin to register authorized merchants with minimal metadata. Maintain merchant whitelist.

**Requirements**:
- [ ] Define merchant data structure (address, name, active status)
- [ ] Implement `register_merchant(admin: Address, merchant: Address, name: String)` function
- [ ] Require admin authorization
- [ ] Store merchant data in contract storage
- [ ] Set merchant as active by default
- [ ] Emit `MerchantRegistered` event

**Proposed Function Signature**:
```rust
pub fn register_merchant(
    env: Env,
    admin: Address,
    merchant: Address,
    name: String,
) -> Result<(), MerchantRegistryError>
```

**Merchant Data Structure**:
```rust
pub struct MerchantInfo {
    pub address: Address,
    pub name: String,
    pub active: bool,
    pub registered_at: u64,  // timestamp
}
```

**Validation Rules**:
- Caller must be admin
- Merchant address must not already be registered
- Name must be non-empty (1-64 characters)

**Events to Emit**:
```rust
Topic: (symbol_short!("MRCHREGD"), merchant)
Data: (name, timestamp)
```

**Related Files**: (To be created)
- `contracts/merchant-registry-contract/src/lib.rs`
- `contracts/merchant-registry-contract/src/storage.rs`
- `contracts/merchant-registry-contract/src/errors.rs`

---

#### ⏳ SC-14: Implement Merchant Validation
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Expose public function to validate if a merchant is active. Consumed by CreditLine contract.

**Requirements**:
- [ ] Implement `is_active_merchant(merchant: Address) -> bool` function
- [ ] No authorization required (public read)
- [ ] Return true only if merchant is registered AND active
- [ ] Optimize for efficient queries

**Proposed Function Signature**:
```rust
pub fn is_active_merchant(env: Env, merchant: Address) -> bool
```

**Validation Logic**:
```rust
pub fn is_active_merchant(env: Env, merchant: Address) -> bool {
    if let Some(merchant_info) = storage::get_merchant(&env, &merchant) {
        merchant_info.active
    } else {
        false
    }
}
```

**Usage in CreditLine Contract**:
```rust
// Before creating loan, validate merchant
let is_valid = merchant_registry.is_active_merchant(&merchant);
if !is_valid {
    panic_with_error!(&env, CreditLineError::InvalidMerchant);
}
```

**Additional Functions** (optional):
- `deactivate_merchant(admin: Address, merchant: Address)` - Admin can suspend merchants
- `get_merchant_info(merchant: Address) -> Option<MerchantInfo>` - Query merchant details

**Related Contracts**:
- Consumed by: CreditLine Contract (loan creation validation)

---

## Phase 6: Liquidity Pool

**Goal**: Implement liquidity provider deposit/withdrawal system and interest distribution.

**Dependencies**: Phase 3 (CreditLine for loan funding/repayment)

**Status**: ⏳ **PENDING**

### Issues

#### ⏳ SC-15: Implement Deposit Liquidity
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Allow investors to deposit funds into liquidity pool. Issue shares representing pool ownership.

**Requirements**:
- [ ] Define pool share data structure
- [ ] Implement `deposit(provider: Address, amount: i128)` function
- [ ] Transfer tokens from provider to pool
- [ ] Calculate and issue shares based on current pool ratio
- [ ] Store provider's share balance
- [ ] Update total pool liquidity
- [ ] Emit `LiquidityDeposited` event

**Proposed Function Signature**:
```rust
pub fn deposit(env: Env, provider: Address, amount: i128) -> u128  // Returns shares issued
```

**Share Calculation**:
```rust
// First depositor sets initial ratio (1:1)
if total_shares == 0 {
    shares_to_issue = amount;
} else {
    // Subsequent deposits: shares proportional to pool value
    shares_to_issue = (amount * total_shares) / total_pool_value;
}
```

**Validation Rules**:
- Amount must be positive
- Provider must have sufficient balance
- Provider must authorize token transfer

**Events to Emit**:
```rust
Topic: (symbol_short!("LIQDEPOS"), provider)
Data: (amount, shares_issued, timestamp)
```

**Related Files**: (To be created)
- `contracts/liquidity-pool-contract/src/lib.rs`
- `contracts/liquidity-pool-contract/src/storage.rs`
- `contracts/liquidity-pool-contract/src/shares.rs`

---

#### ⏳ SC-16: Implement Withdraw Liquidity
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Allow liquidity providers to withdraw funds based on their share ownership. Validate available liquidity.

**Requirements**:
- [ ] Implement `withdraw(provider: Address, shares: u128)` function
- [ ] Calculate withdrawal amount based on share ratio
- [ ] Validate provider has sufficient shares
- [ ] Validate pool has sufficient available liquidity
- [ ] Burn provider's shares
- [ ] Transfer tokens to provider
- [ ] Update total pool liquidity
- [ ] Emit `LiquidityWithdrawn` event

**Proposed Function Signature**:
```rust
pub fn withdraw(env: Env, provider: Address, shares: u128) -> i128  // Returns amount withdrawn
```

**Withdrawal Calculation**:
```rust
withdrawal_amount = (shares * total_pool_value) / total_shares;
available_liquidity = total_pool_value - total_loaned;

if withdrawal_amount > available_liquidity {
    panic_with_error!(&env, LiquidityPoolError::InsufficientLiquidity);
}
```

**Validation Rules**:
- Provider must have sufficient shares
- Pool must have sufficient available (non-loaned) liquidity
- Shares to burn must be positive

**Events to Emit**:
```rust
Topic: (symbol_short!("LIQWDRAW"), provider)
Data: (amount, shares_burned, timestamp)
```

**Edge Cases**:
- Partial withdrawals allowed (withdraw some but not all shares)
- If pool is fully utilized (all funds loaned), withdrawals must wait

---

#### ⏳ SC-17: Implement Interest Distribution
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Distribute interest from loan repayments to liquidity providers based on share ownership.

**Requirements**:
- [ ] Track interest accumulated from loan repayments
- [ ] Update pool value when interest received
- [ ] Interest automatically reflected in share value (no explicit distribution needed)
- [ ] Implement `get_share_value() -> i128` function for transparency
- [ ] Optional: Implement `claim_interest(provider: Address)` for explicit claims
- [ ] Emit `InterestDistributed` event

**Proposed Function Signature**:
```rust
pub fn get_share_value(env: Env) -> i128  // Returns current value of 1 share

pub fn receive_interest(env: Env, amount: i128)  // Called by CreditLine contract
```

**Interest Accumulation**:
```rust
// When CreditLine repays loan with interest
pub fn receive_interest(env: Env, amount: i128) {
    total_pool_value += amount;
    // Share value automatically increases for all holders
}

// Share value calculation
share_value = total_pool_value / total_shares;
```

**Fee Distribution** (optional):
```rust
// Example: 85% to LPs, 10% protocol fee, 5% merchant incentives
interest_to_lps = interest_received * 0.85;
protocol_fee = interest_received * 0.10;
merchant_incentive = interest_received * 0.05;
```

**Events to Emit**:
```rust
Topic: (symbol_short!("INTRDIST"),)
Data: (total_interest, new_share_value, timestamp)
```

**Related Contracts**:
- Called by: CreditLine Contract (loan repayment processing)

---

## Phase 7: Contract Tests

**Goal**: Comprehensive test coverage for all contracts and integration scenarios.

**Dependencies**: All previous phases

**Status**: 🚧 **IN PROGRESS** (Reputation tests complete)

### Issues

#### ✅ SC-18: Unit Tests for Reputation Contract
**Status**: Completed
**Implementation**: [contracts/reputation-contract/src/tests.rs](../contracts/reputation-contract/src/tests.rs)

**Description**:
Comprehensive unit tests covering admin, updaters, score operations, and edge cases.

**Requirements**:
- [x] Test admin initialization and transfer
- [x] Test updater registration and authorization
- [x] Test score increase/decrease operations
- [x] Test bounds validation (0-100 range)
- [x] Test overflow/underflow prevention
- [x] Test unauthorized access attempts
- [x] Test event emission
- [x] Achieve >90% code coverage

**Implemented Tests** (10 tests):

1. ✅ `it_sets_admin` - Admin initialization
2. ✅ `it_gets_admin` - Admin retrieval
3. ✅ `it_sets_updater` - Updater registration
4. ✅ `it_checks_updater` - Updater status check
5. ✅ `it_gets_score` - Score retrieval (with default)
6. ✅ `it_increases_score` - Score increment
7. ✅ `it_decreases_score` - Score decrement
8. ✅ `it_sets_score` - Direct score setting
9. ✅ `it_prevents_unauthorized_updates` - Authorization validation
10. ✅ `it_enforces_score_bounds` - Bounds validation
11. ✅ `it_gets_version` - Version retrieval

**Test Coverage**:
- Function coverage: 100%
- Authorization paths: 100%
- Error conditions: 100%
- Edge cases: Covered (min/max scores, overflow/underflow)

**Related Files**:
- `contracts/reputation-contract/src/tests.rs`

---

#### ⏳ SC-19: Unit Tests for CreditLine Contract
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Comprehensive unit tests for loan creation, repayment, and default flows. Validate state transitions.

**Requirements**:
- [ ] Test loan creation with valid parameters
- [ ] Test loan creation with invalid inputs (insufficient guarantee, invalid merchant)
- [ ] Test partial loan repayment
- [ ] Test full loan repayment
- [ ] Test loan default marking
- [ ] Test unauthorized actions
- [ ] Test state transition validation (Active → Paid, Active → Defaulted)
- [ ] Test integration with Reputation contract (score updates)
- [ ] Test integration with Merchant Registry (merchant validation)
- [ ] Test integration with Liquidity Pool (fund transfers)
- [ ] Achieve >90% code coverage

**Proposed Test Cases**:

1. **Loan Creation Tests**:
   - `test_create_loan_success` - Happy path
   - `test_create_loan_insufficient_guarantee` - Validation error
   - `test_create_loan_invalid_merchant` - Merchant check fails
   - `test_create_loan_insufficient_liquidity` - Pool has no funds

2. **Repayment Tests**:
   - `test_repay_partial` - Partial payment
   - `test_repay_full` - Full payment, loan marked paid
   - `test_repay_unauthorized` - Non-borrower tries to repay
   - `test_repay_overpayment` - Payment exceeds remaining balance

3. **Default Tests**:
   - `test_mark_defaulted_overdue` - Mark loan defaulted when overdue
   - `test_mark_defaulted_early` - Cannot default loan before due date
   - `test_default_reputation_decrease` - Verify reputation penalty

4. **State Transition Tests**:
   - `test_cannot_repay_defaulted_loan` - No repayment after default
   - `test_cannot_repay_paid_loan` - No repayment after full payment

**Related Files**: (To be created)
- `contracts/creditline-contract/src/tests.rs`

---

#### ⏳ SC-20: Unit Tests for Liquidity Pool
**Status**: Pending
**Assigned**: Unassigned

**Description**:
Comprehensive tests for deposit, withdrawal, and interest distribution. Cover low liquidity scenarios.

**Requirements**:
- [ ] Test initial deposit (share issuance)
- [ ] Test subsequent deposits (share calculation)
- [ ] Test withdrawal with sufficient liquidity
- [ ] Test withdrawal with insufficient liquidity (error)
- [ ] Test interest accumulation and share value increase
- [ ] Test loan funding (liquidity allocation)
- [ ] Test repayment processing (liquidity return)
- [ ] Test edge case: withdraw all liquidity
- [ ] Test edge case: pool fully utilized (no available liquidity)
- [ ] Achieve >90% code coverage

**Proposed Test Cases**:

1. **Deposit Tests**:
   - `test_first_deposit` - Initial LP, 1:1 share ratio
   - `test_subsequent_deposit` - Share calculation based on pool value
   - `test_deposit_after_interest` - Share value reflects interest

2. **Withdrawal Tests**:
   - `test_withdraw_partial` - Withdraw some shares
   - `test_withdraw_full` - Withdraw all shares
   - `test_withdraw_insufficient_liquidity` - Pool fully loaned, withdrawal fails
   - `test_withdraw_too_many_shares` - LP tries to withdraw more than owned

3. **Interest Tests**:
   - `test_interest_distribution` - Share value increases with interest
   - `test_multiple_lps_interest` - Interest fairly distributed
   - `test_share_value_calculation` - Verify share value formula

4. **Integration Tests**:
   - `test_loan_funding_reduces_available_liquidity`
   - `test_repayment_increases_available_liquidity`
   - `test_withdrawal_blocked_when_pool_utilized`

**Related Files**: (To be created)
- `contracts/liquidity-pool-contract/src/tests.rs`

---

## Progress Tracking

### Completed Milestones
- ✅ **Reputation Contract MVP** - Fully functional with tests (SC-01 through SC-07, SC-18)
- ✅ **CI/CD Pipeline** - GitHub Actions workflow for build and test

### Current Focus
- 🎯 **Phase 3: CreditLine Core** - Next major milestone

### Upcoming Milestones
1. CreditLine Contract implementation (SC-08, SC-09, SC-10)
2. CreditLine ↔ Reputation integration (SC-11, SC-12)
3. Merchant Registry implementation (SC-13, SC-14)
4. Liquidity Pool implementation (SC-15, SC-16, SC-17)
5. Complete test coverage (SC-19, SC-20)

### Long-term Goals
- Smart contract auditing
- Testnet deployment
- Mainnet deployment
- Frontend integration
- Governance token launch

---

## Development Standards

### Issue Workflow
1. **Planning**: Define requirements and acceptance criteria
2. **Implementation**: Write contract code following module pattern
3. **Testing**: Write comprehensive unit tests
4. **Review**: Code review and security audit
5. **Deployment**: Deploy to testnet, then mainnet

### Branch Strategy
- `main` - Production-ready code
- `develop` - Integration branch
- `feat/<issue-id>-<description>` - Feature branches (e.g., `feat/SC-08-loan-creation`)
- `fix/<issue-id>-<description>` - Bug fix branches

### Commit Standards
Follow Conventional Commits:
```
feat: implement loan creation (SC-08)
fix: prevent overflow in score calculation (SC-06)
test: add boundary tests for reputation (SC-18)
docs: update roadmap with completed issues
```

### Definition of Done
- [ ] Code implemented and follows module pattern
- [ ] Unit tests written with >90% coverage
- [ ] Documentation updated (README, inline comments)
- [ ] Code reviewed by at least one other developer
- [ ] CI/CD pipeline passes (build + test)
- [ ] Roadmap updated with completion status

---

## Questions or Feedback?

If you have questions about the roadmap or want to contribute:
1. Check existing GitHub issues
2. Review [Contributing Guide](CONTRIBUTING.md)
3. Open a new issue for discussion
4. Join our community chat (link TBD)

---

**Last Updated**: 2026-01-15
**Maintained By**: TrustUp Core Team
