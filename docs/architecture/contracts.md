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

## Adapter Trustless Contract ✅

**Status**: Implemented and tested

**Purpose**: Remove the single-admin trust assumption from privileged, cross-contract
operations. Each TrustUp contract gates sensitive operations (changing an admin,
updating a cross-contract address, tuning risk parameters) behind a single `admin`
address — a single point of failure and of trust. The adapter is a generic **M-of-N
multi-sig + timelock gateway**: instead of an admin calling a privileged function
directly, a signer proposes the call, a threshold of signers approve it, and only
after a configured timelock delay can anyone trigger its execution. No single
signer, including the adapter's own admin, can execute a privileged call alone.

### Architecture

**State**:
```rust
pub enum DataKey {
    Admin,              // Instance: Admin address
    Signers,            // Instance: Vec<Address> of authorized signers
    Threshold,          // Instance: u32 approvals required to execute
    TimelockSecs,       // Instance: u64 delay (seconds) between proposal and execution
    NextActionId,       // Instance: u64 counter
    Action(u64),        // Persistent: ActionId → Action
}

pub struct Action {
    pub id: u64,
    pub target: Address,        // Contract to call
    pub function: Symbol,       // Function to invoke
    pub args: Vec<Val>,         // Call arguments
    pub proposer: Address,
    pub proposed_at: u64,       // Ledger timestamp of proposal
    pub approvals: Vec<Address>,
    pub executed: bool,
    pub canceled: bool,
}
```

**Public API**:
```rust
// Setup
pub fn initialize(env: Env, admin: Address, signers: Vec<Address>, threshold: u32, timelock_secs: u64)

// Proposal lifecycle
pub fn propose_action(env: Env, proposer: Address, target: Address, function: Symbol, args: Vec<Val>) -> u64
pub fn approve_action(env: Env, signer: Address, action_id: u64)
pub fn revoke_approval(env: Env, signer: Address, action_id: u64)
pub fn execute_action(env: Env, caller: Address, action_id: u64) -> Val
pub fn cancel_action(env: Env, admin: Address, action_id: u64)

// Signer / config management (admin only)
pub fn add_signer(env: Env, admin: Address, signer: Address)
pub fn remove_signer(env: Env, admin: Address, signer: Address)
pub fn set_threshold(env: Env, admin: Address, threshold: u32)
pub fn set_timelock(env: Env, admin: Address, timelock_secs: u64)

// Queries
pub fn get_action(env: Env, action_id: u64) -> Action
pub fn is_approved(env: Env, action_id: u64) -> bool
pub fn is_executable(env: Env, action_id: u64) -> bool
pub fn get_signers(env: Env) -> Vec<Address>
pub fn get_threshold(env: Env) -> u32
pub fn get_timelock(env: Env) -> u64
pub fn get_admin(env: Env) -> Address
```

**Business Logic**:

1. **Proposal**: Any authorized signer proposes a target contract, function, and
   argument list. The proposer's approval is recorded immediately.
2. **Approval**: Other signers approve; each signer may approve once and may revoke
   their approval any time before execution.
3. **Execution**: Once approvals reach `threshold` **and** `proposed_at + timelock`
   has elapsed, any address can call `execute_action`, which invokes the target
   contract via `Env::invoke_contract` and marks the action executed.
4. **Cancellation**: The admin can cancel a pending action, which blocks execution
   permanently — used to reject inappropriate proposals without disabling the
   whole adapter.

**Events**:
- `ACTNPROP` / `ACTNAPPRV` / `ACTNREVK` / `ACTNEXEC` / `ACTNCANC`: Action lifecycle
- `SIGNERADD` / `SIGNERRM`: Signer set changes
- `THRESHCHG` / `TMLOCKCHG`: Config changes
- `ADMINCHGD`: Admin changed

**Access Control**:
- Signers: Can propose, approve, and revoke their own approval
- Admin: Can cancel actions and manage signers/threshold/timelock
- Public: Can execute any action that has met threshold + timelock, and can read
  any action

**Security Features**:
- Threshold enforced against the current signer count on every config change
  (`remove_signer` and `set_threshold` both re-validate the invariant)
- Timelock computed with `saturating_add` to avoid overflow
- Execution and cancellation are idempotent-safe (`executed`/`canceled` flags)
- Event emission for every state change, for off-chain auditability

**What it does *not* do**: it does not replace each contract's own `admin` field —
contracts must be reconfigured to name the adapter (or an address it controls) as
their admin/updater to have its guarantees enforced. It is also a generic relay: it
does not interpret or validate the semantics of the calls it forwards.

---

## Parameters Contract ✅

**Status**: Implemented and tested

**Purpose**: Hold `ProtocolParameters` (guarantee/reputation/interest/penalty
settings) and, since this extension, govern changes to them and to a protocol-wide
emergency pause flag through proposal + multi-sig approval + timelock, rather than
a single admin applying changes instantly. This replaces the "Phase 3: Governance
token for protocol parameters" line item in the roadmap with a lighter-weight
multi-sig/timelock design instead of a separate token.

### Architecture

**State** (in addition to the pre-existing `admin` / `params` instance keys):
```rust
pub enum DataKey {
    Signers,          // Instance: Vec<Address> of governance signers
    Threshold,        // Instance: u32 approvals required to execute
    TimelockSecs,     // Instance: u64 delay for parameter-change proposals
    NextProposalId,   // Instance: u64 counter
    Proposal(u64),    // Persistent: ProposalId → Proposal
    Paused,           // Instance: bool, protocol-wide emergency pause flag
}

pub enum ProposalKind {
    UpdateParameters(ProtocolParameters),
    SetPaused(bool),
}

pub struct Proposal {
    pub id: u64,
    pub kind: ProposalKind,
    pub proposer: Address,
    pub proposed_at: u64,
    pub approvals: Vec<Address>,
    pub status: ProposalStatus, // Pending | Executed | Cancelled
}
```

**Public API** (governance additions):
```rust
// One-time migration off the single hardcoded admin (admin-only)
pub fn migrate_to_multisig(env: Env, admin: Address, signers: Vec<Address>, threshold: u32, timelock_secs: u64)

// Proposal lifecycle
pub fn propose_parameters(env: Env, proposer: Address, params: ProtocolParameters) -> u64
pub fn propose_pause(env: Env, proposer: Address, paused: bool) -> u64
pub fn approve_proposal(env: Env, signer: Address, proposal_id: u64)
pub fn execute_proposal(env: Env, caller: Address, proposal_id: u64)
pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u64)

// Queries
pub fn is_paused(env: Env) -> bool
pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal
pub fn get_signers(env: Env) -> Vec<Address>
pub fn get_threshold(env: Env) -> u32
pub fn get_timelock(env: Env) -> u64
```

**Business Logic**:

1. **Migration**: The current single admin calls `migrate_to_multisig` once to
   install a signer set, an approval threshold, and a timelock (seconds). After
   this, the legacy `update_parameters` and `set_admin` entrypoints are disabled
   (`GovernanceActive` error) — all further changes must go through proposals.
2. **Proposing**: Any signer proposes either a new `ProtocolParameters` value or a
   pause/unpause toggle. The proposer's own approval is recorded immediately.
3. **Approving**: Other signers approve once each; a proposal becomes executable
   once approvals reach `threshold`.
4. **Executing**: `UpdateParameters` proposals additionally require
   `proposed_at + timelock_secs` to have elapsed before they can execute, giving
   the community time to react to a pending change. `SetPaused` proposals skip
   the timelock so an emergency pause isn't delayed — they still require the
   full signer threshold, so pausing can't be triggered unilaterally.
5. **Cancelling**: The original proposer or the (legacy) admin can withdraw a
   pending proposal at any time before execution.

**Events**: `PRMPROP` / `PRMAPPR` / `PRMEXEC` / `PRMCANC` (proposal lifecycle),
`PRMPAUS` / `PRMUNPAU` (pause toggled), `PRMMIGR` (migrated to multi-sig),
`PARMUPDT` / `PARMADMN` (pre-existing parameter/admin-update events, still emitted
on execution of an `UpdateParameters` proposal).

**Access Control**:
- Signers: Can propose, approve, and (if they proposed it) cancel
- Admin (pre-migration only): Can call the legacy instant-update entrypoints;
  post-migration, can still cancel any pending proposal
- Public: Can execute any proposal that has met threshold (+ timelock, for
  parameter changes), and can read any proposal or the paused flag

**Security Features**:
- Threshold validated against signer count at migration time
- Timelock applied only to parameter changes, computed with `saturating_add`
- Execution/cancellation are idempotent-safe via `ProposalStatus`
- Legacy single-admin update path is hard-disabled once governance is active,
  so there is no unguarded fallback once a protocol has migrated

**What it does *not* do**: it does not itself gate other contracts' functions —
contracts that want to honor the pause flag must call `is_paused` (as
`liquidity-pool-contract`'s own `pause`/`unpause` did previously with its local
flag) or be wired through the `adapter-trustless-contract` for cross-contract
enforcement.

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
- [Adapter Trustless Contract Source](../../contracts/adapter-trustless-contract/src/lib.rs)
- [Roadmap](../ROADMAP.md) - Implementation timeline
- [Storage Patterns](storage-patterns.md) - Storage best practices
