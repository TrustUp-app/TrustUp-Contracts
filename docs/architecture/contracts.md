# Contract Architecture Details

Detailed architecture for each smart contract in the TrustUp system.

## Reputation Contract ✅

**Status**: Implemented and tested

**Purpose**: Track and manage user credit scores (0-100)

### Architecture

**State**:
```rust
pub enum DataKey {
    Admin,              // Instance: Admin address
    Score(Address),     // Persistent: User scores (Address → u32)
    Updater(Address),   // Persistent: Authorized updaters (Address → bool)
}
```

**Public API**:
```rust
// Admin functions
pub fn initialize(env: Env, admin: Address)
pub fn set_admin(env: Env, admin: Address, new_admin: Address)
pub fn set_updater(env: Env, admin: Address, updater: Address, allowed: bool)

// Score operations
pub fn get_score(env: Env, user: Address) -> u32
pub fn set_score(env: Env, updater: Address, user: Address, score: u32)
pub fn increase_score(env: Env, updater: Address, user: Address, amount: u32)
pub fn decrease_score(env: Env, updater: Address, user: Address, amount: u32)
```

**Events**:
- `SCORECHGD`: Score changed (user, old_score, new_score, reason)
- `UPDCHGD`: Updater status changed (updater, allowed)
- `ADMINCHGD`: Admin changed (old_admin, new_admin)

**Access Control**:
- Admin: Can set updaters and transfer admin
- Updaters: Can modify user scores
- Public: Can read any score

**Security Features**:
- Checked arithmetic (overflow/underflow protection)
- Range validation (0-100)
- Authorization before state changes
- Event emission for auditability

---

## CreditLine Contract ✅

**Status**: Implemented and tested

**Purpose**: Manage loan creation, repayment, defaults, and late fees

### Architecture

**Loan status**:
```rust
pub enum LoanStatus {
    Pending,
    Active,
    Repaid,
    Defaulted,
    Cancelled,
}
```

**Public API**:
```rust
// Admin / configuration
pub fn initialize(env: Env, admin: Address, token: Address, reputation_contract: Address)
pub fn set_reputation_contract(env: Env, admin: Address, address: Address)
pub fn set_merchant_registry(env: Env, admin: Address, address: Address)
pub fn set_liquidity_pool(env: Env, admin: Address, address: Address)
pub fn set_parameters_contract(env: Env, admin: Address, address: Address)

// Loan operations
pub fn create_loan(
    env: Env,
    user: Address,
    merchant: Address,
    total_amount: i128,
    guarantee_amount: i128,
    repayment_schedule: Vec<RepaymentInstallment>,
) -> u64

pub fn request_loan(...) -> u64
pub fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) -> i128
pub fn cancel_loan(env: Env, caller: Address, loan_id: u64)
pub fn mark_defaulted(env: Env, loan_id: u64) -> Result<(), CreditLineError>
pub fn apply_late_fees(env: Env, loan_id: u64)
pub fn warn_grace_period(env: Env, loan_id: u64) -> Result<(), CreditLineError>

// Queries
pub fn get_loan(env: Env, loan_id: u64) -> Loan
pub fn get_user_loans(env: Env, borrower: Address, start: u64, limit: u32) -> Vec<Loan>
pub fn get_user_loan_count(env: Env, borrower: Address) -> u64
pub fn get_user_active_debt(env: Env, borrower: Address) -> i128
```

**Business Logic**:

1. **Loan Creation**:
   - Validate merchant is active (MerchantRegistry)
   - Check borrower reputation and liquidity availability
   - Fund loan from Liquidity Pool and store loan as `Active`
   - Emit loan created event

2. **Repayment** (`repay_loan`):
   - Validate loan exists and is `Active`
   - Validate borrower authorization and repayment amount
   - Accrue outstanding late fees before applying payment
   - Apply payment priority: principal → interest → service fee → late fees
   - Transfer repayment to Liquidity Pool via `receive_repayment`
   - If fully repaid:
     - Transition loan to `Repaid`
     - Refund guarantee to borrower
     - Increase reputation score (+10 on time, +15 early)
   - Emit `LoanRepaid` event (`LOANRPD`)

3. **Default**:
   - Validate loan is overdue and still `Active`
   - Transfer guarantee to Liquidity Pool
   - Mark loan as `Defaulted` and decrease reputation score
   - Emit default event

**Cross-Contract Interactions**:
```
create_loan:
    → MerchantRegistry.is_active(merchant) ✓
    → Reputation.get_score(borrower) → rate
    → LiquidityPool.fund_loan(amount) → funds
    → Transfer funds to merchant

repay_loan:
    → LiquidityPool.receive_repayment(amount)
    → Reputation.increase_score(borrower, +10 or +15)

mark_defaulted:
    → LiquidityPool.receive_guarantee(guarantee)
    → Reputation.decrease_score(borrower, -30)
```

**Security Considerations**:
- Reentrancy protection (state changes before external calls)
- Validation of all inputs
- Authorization checks
- Safe arithmetic
- Status machine (prevent invalid state transitions)

---

## Merchant Registry Contract ⏳

**Status**: Planned

**Purpose**: Whitelist of authorized merchants

### Planned Architecture

**State**:
```rust
pub enum DataKey {
    // Instance storage
    Admin,

    // Persistent storage
    Merchant(Address),        // Address → MerchantInfo
    MerchantCount,            // Total merchant count
}

#[contracttype]
pub struct MerchantInfo {
    pub name: String,
    pub registration_date: u64,
    pub active: bool,
    pub total_sales: u64,
}
```

**Public API** (planned):
```rust
// Admin functions
pub fn initialize(env: Env, admin: Address)
pub fn register_merchant(env: Env, admin: Address, merchant: Address, name: String)
pub fn deactivate_merchant(env: Env, admin: Address, merchant: Address)
pub fn activate_merchant(env: Env, admin: Address, merchant: Address)

// Queries
pub fn is_active(env: Env, merchant: Address) -> bool
pub fn get_merchant_info(env: Env, merchant: Address) -> MerchantInfo
pub fn get_merchant_count(env: Env) -> u64
```

**Business Logic**:

1. **Registration**:
   - Admin-only
   - Validate merchant not already registered
   - Validate name (non-empty, length limits)
   - Store merchant info
   - Set active = true
   - Emit registration event

2. **Activation/Deactivation**:
   - Admin-only
   - Toggle active status
   - Emit status change event

3. **Validation**:
   - Called by CreditLine before loan creation
   - Fast boolean check

**Events**:
- `MERCHTREG`: Merchant registered
- `MERCHTSTATUS`: Merchant status changed

---

## Liquidity Pool Contract ⏳

**Status**: Planned

**Purpose**: Manage liquidity provider deposits, withdrawals, and loan funding

### Planned Architecture

**State**:
```rust
pub enum DataKey {
    // Instance storage
    Admin,
    TokenAddress,           // Asset being pooled (e.g., USDC)
    TotalShares,            // Total LP shares issued
    TotalLiquidity,         // Total tokens in pool
    LockedLiquidity,        // Tokens in active loans

    // Persistent storage
    LpShares(Address),      // Address → shares owned
}

#[contracttype]
pub struct PoolStats {
    pub total_liquidity: i128,
    pub locked_liquidity: i128,
    pub available_liquidity: i128,
    pub total_shares: i128,
    pub share_price: i128,  // In basis points
}
```

**Public API** (planned):
```rust
// Initialization
pub fn initialize(env: Env, admin: Address, token: Address)

// LP operations
pub fn deposit(env: Env, provider: Address, amount: i128) -> i128  // Returns shares
pub fn withdraw(env: Env, provider: Address, shares: i128) -> i128  // Returns amount

// CreditLine operations (restricted)
pub fn fund_loan(env: Env, creditline: Address, amount: i128)
pub fn receive_repayment(env: Env, creditline: Address, amount: i128)
pub fn receive_guarantee(env: Env, creditline: Address, amount: i128)

// Queries
pub fn get_pool_stats(env: Env) -> PoolStats
pub fn get_lp_shares(env: Env, provider: Address) -> i128
pub fn calculate_withdrawal(env: Env, shares: i128) -> i128
```

**Share Mechanics**:

**First deposit**:
```
shares = amount
share_price = 1.0
```

**Subsequent deposits**:
```
shares = (amount × total_shares) / total_pool_value
```

**Withdrawal**:
```
amount = (shares × total_pool_value) / total_shares
```

**Share value increases** as interest accumulates:
```
initial: 1 share = $1.00
after 1 year (8% APY): 1 share = $1.08
```

**Business Logic**:

1. **Deposit**:
   - Transfer tokens from provider
   - Calculate shares to issue
   - Mint shares
   - Update totals
   - Emit deposit event

2. **Withdraw**:
   - Validate sufficient shares
   - Validate sufficient available liquidity
   - Calculate withdrawal amount
   - Burn shares
   - Transfer tokens to provider
   - Update totals
   - Emit withdrawal event

3. **Fund Loan**:
   - Only CreditLine can call
   - Validate sufficient available liquidity
   - Transfer tokens to merchant
   - Increase locked_liquidity
   - Emit loan funded event

4. **Receive Repayment**:
   - Only CreditLine can call
   - Receive tokens (principal + interest)
   - Decrease locked_liquidity
   - Interest increases total_liquidity (share value increases)
   - Emit repayment received event

5. **Receive Guarantee**:
   - Only CreditLine can call (on default)
   - Receive forfeited guarantee
   - Offsets loss from defaulted loan
   - Emit guarantee received event

**Security Considerations**:
- Reentrancy protection
- Share calculation overflow protection
- Minimum share amounts (prevent rounding exploits)
- Access control (only CreditLine can fund/repay)
- Liquidity checks before withdrawals

---

## Contract Interactions

### Create Loan Flow

```
User → CreditLine.create_loan()
    ├─→ MerchantRegistry.is_active() → bool
    ├─→ Reputation.get_score() → score
    │   └─→ Calculate interest rate from score
    ├─→ LiquidityPool.fund_loan() → funds
    │   └─→ Token.transfer(pool, merchant, amount)
    └─→ Emit LoanCreated event
```

### Repayment Flow

```
User → CreditLine.repay_loan()
    ├─→ Token.transfer(user, pool, amount)
    ├─→ LiquidityPool.receive_repayment()
    ├─→ If fully repaid:
    │   ├─→ Token.transfer(pool, user, guarantee)
    │   └─→ Reputation.increase_score(+10)
    └─→ Emit LoanRepaid event
```

### Default Flow

```
Anyone → CreditLine.mark_defaulted()
    ├─→ Validate loan is overdue
    ├─→ LiquidityPool.receive_guarantee()
    ├─→ Reputation.decrease_score(-30)
    └─→ Emit LoanDefaulted event
```

## Upgrade Strategy

All contracts follow upgrade pattern:
1. Deploy new version
2. Migrate state if needed
3. Update references in other contracts
4. Emit upgrade event
5. Deprecate old version

## Testing Strategy

### Unit Tests
- Individual function logic
- Error conditions
- Boundary values
- Access control

### Integration Tests
- Cross-contract interactions
- Full loan lifecycle
- Complex scenarios
- Error propagation

### Invariant Tests
- Score range (0-100)
- Share conservation (total shares = sum of individual shares)
- Liquidity conservation (total = available + locked)
- No negative balances

## Resources

- [Reputation Contract Source](../../contracts/reputation-contract/src/lib.rs)
- [Roadmap](../ROADMAP.md) - Implementation timeline
- [Storage Patterns](storage-patterns.md) - Storage best practices

---

## Adapter Trustless Contract ✅

**Status**: Implemented and tested

**Purpose**: Trustless escrow adapter that holds the borrower's 20% guarantee deposit for the lifetime of a loan and releases or seizes it based on loan outcome.

### Architecture

**State**:
```rust
// Instance storage
Admin      → Address
CreditLine → Address
Token      → Address

// Persistent storage
Escrows    → Map<u64, EscrowEntry>
```

```rust
#[contracttype]
pub struct EscrowEntry {
    pub borrower: Address,
    pub amount: i128,
    pub status: EscrowStatus,  // Locked | Released | Seized
}
```

**Public API**:
```rust
// Initialisation
pub fn initialize(env: Env, admin: Address, creditline: Address, token: Address)

// Admin
pub fn set_admin(env: Env, new_admin: Address)
pub fn set_creditline(env: Env, admin: Address, creditline: Address)

// Escrow lifecycle (CreditLine-only)
pub fn lock_guarantee(env: Env, creditline: Address, loan_id: u64, borrower: Address, amount: i128)
pub fn release_guarantee(env: Env, creditline: Address, loan_id: u64)
pub fn seize_guarantee(env: Env, creditline: Address, loan_id: u64, pool: Address)

// Queries
pub fn get_escrow(env: Env, loan_id: u64) -> EscrowEntry
pub fn get_admin(env: Env) -> Address
pub fn get_creditline(env: Env) -> Address
```

**Events**:
- `ESCRWLCK`: Guarantee locked (`loan_id`, `borrower`, `amount`)
- `ESCRWRLS`: Guarantee released to borrower (`loan_id`, `borrower`, `amount`)
- `ESCRWSZD`: Guarantee seized to liquidity pool (`loan_id`, `pool`, `amount`)

**Access Control**:

| Caller     | Allowed operations                                                |
|------------|-------------------------------------------------------------------|
| Admin      | `set_admin`, `set_creditline`                                     |
| CreditLine | `lock_guarantee`, `release_guarantee`, `seize_guarantee`          |
| Anyone     | `get_escrow`, `get_admin`, `get_creditline`                       |

**Escrow Status Machine**:
```
(none) ──lock_guarantee──→ Locked
Locked ──release_guarantee──→ Released   (repayment success)
Locked ──seize_guarantee──→ Seized       (default)
```

**Integration**:
```
CreditLine.create_loan()
    └─→ AdapterTrustless.lock_guarantee(loan_id, borrower, guarantee)
            └─→ Token.transfer(borrower → adapter)

CreditLine.repay_loan() [fully repaid]
    └─→ AdapterTrustless.release_guarantee(loan_id)
            └─→ Token.transfer(adapter → borrower)

CreditLine.mark_defaulted()
    └─→ AdapterTrustless.seize_guarantee(loan_id, pool)
            └─→ Token.transfer(adapter → liquidity_pool)
```

**Design document**: [adapter-trustless-contract-design.md](../adapter-trustless-contract-design.md)
