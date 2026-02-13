# Storage Patterns

Storage strategies and patterns for Soroban smart contracts in TrustUp.

## Soroban Storage Types

### Instance Storage

**Best for**: Contract configuration and contract-level state

**Characteristics**:
- Lives with the contract instance
- Relatively cheap
- Persists unless contract is deleted
- Used for admin addresses, global config

**Example**:
```rust
pub enum DataKey {
    Admin,      // Single admin address
    Config,     // Contract configuration
}

// Write
env.storage().instance().set(&DataKey::Admin, &admin_address);

// Read
let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
```

**TrustUp Usage**:
- Reputation: Admin address
- CreditLine: Configuration parameters
- Merchant Registry: Admin address

### Persistent Storage

**Best for**: Critical data that must never expire

**Characteristics**:
- Never expires (unless explicitly removed)
- More expensive than temporary
- Required for financial/critical data
- Explicitly extendable TTL

**Example**:
```rust
pub enum DataKey {
    Score(Address),     // User reputation scores
    Loan(u64),          // Loan records
    Balance(Address),   // User balances
}

// Write
env.storage().persistent().set(&DataKey::Score(user), &score);

// Read
let score: u32 = env.storage().persistent()
    .get(&DataKey::Score(user))
    .unwrap_or(DEFAULT_SCORE);

// Extend TTL
env.storage().persistent().extend_ttl(&DataKey::Score(user), 100_000, 100_000);
```

**TrustUp Usage**:
- Reputation: User scores (critical, long-term)
- CreditLine: Active loans
- Liquidity Pool: LP shares and deposits

### Temporary Storage

**Best for**: Caches, temporary computations, non-critical data

**Characteristics**:
- Expires after ~1 day by default
- Cheaper than persistent
- Good for caching, temp calculations
- Must be refreshed periodically

**Example**:
```rust
pub enum DataKey {
    CachedRate,         // Interest rate cache
    TempLookup(u64),    // Temporary lookup table
}

// Write (with TTL)
env.storage().temporary().set(&DataKey::CachedRate, &rate);

// Read
let rate: u32 = env.storage().temporary()
    .get(&DataKey::CachedRate)
    .unwrap_or_else(|| calculate_rate());

// Extend if needed
env.storage().temporary().extend_ttl(&DataKey::CachedRate, 50_000, 50_000);
```

**TrustUp Usage**:
- CreditLine: Cached interest rates
- Merchant Registry: Temporary merchant status lookups

## DataKey Patterns

### Singleton Keys

For single values:
```rust
pub enum DataKey {
    Admin,              // One admin
    TotalSupply,        // One total
    ContractVersion,    // One version
}
```

### Map Keys

For key-value mappings:
```rust
pub enum DataKey {
    Score(Address),           // Address → Score
    Loan(u64),               // LoanId → Loan
    LpShares(Address),       // Address → Shares
    Updater(Address),        // Address → bool
}
```

### Composite Keys

For multi-level mappings:
```rust
pub enum DataKey {
    UserLoan(Address, u64),           // (User, LoanId) → Loan
    MerchantSale(Address, u64),       // (Merchant, SaleId) → Sale
    PoolAllocation(Address, Address), // (Pool, Token) → Amount
}
```

## TrustUp Storage Design

### Reputation Contract

```rust
pub enum DataKey {
    // Instance storage
    Admin,                  // Admin address (singleton)

    // Persistent storage
    Score(Address),         // User scores (Address → u32)

    // Map storage
    Updater(Address),       // Authorized updaters (Address → bool)
}
```

**Rationale**:
- Admin: Rarely changes, instance-level
- Scores: Critical data, must persist
- Updaters: Authorization data, persistent

### CreditLine Contract (Planned)

```rust
pub enum DataKey {
    // Instance storage
    Admin,
    ReputationContract,
    PoolContract,

    // Persistent storage
    Loan(u64),                      // LoanId → Loan
    UserLoans(Address),             // Address → Vec<u64>
    NextLoanId,                     // Counter

    // Temporary storage (optional)
    CachedInterestRate(Address),    // Address → u32
}
```

**Rationale**:
- Loans: Financial data, must persist
- User loan list: Critical for user tracking
- Interest rates: Can be cached temporarily

### Merchant Registry (Planned)

```rust
pub enum DataKey {
    // Instance storage
    Admin,

    // Persistent storage
    Merchant(Address),      // Address → MerchantInfo
    MerchantActive(Address), // Address → bool
}
```

### Liquidity Pool (Planned)

```rust
pub enum DataKey {
    // Instance storage
    TotalShares,
    TotalLiquidity,

    // Persistent storage
    LpShares(Address),      // Address → u64
    LockedAmount,           // Amount in active loans
}
```

## Access Patterns

### Read Pattern

```rust
pub fn get_score(env: &Env, user: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Score(user.clone()))
        .unwrap_or(DEFAULT_SCORE)
}
```

**Best Practice**: Always provide default or handle missing keys gracefully

### Write Pattern

```rust
pub fn set_score(env: &Env, user: &Address, score: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::Score(user.clone()), &score);
}
```

**Best Practice**: Validate before writing, emit events after writing

### Update Pattern

```rust
pub fn increase_score(env: &Env, user: &Address, amount: u32) -> u32 {
    let current = get_score(env, user);
    let new_score = current.checked_add(amount).unwrap();

    // Validate
    if new_score > MAX_SCORE {
        panic_with_error!(env, Error::OutOfBounds);
    }

    // Update
    set_score(env, user, new_score);

    // Event
    emit_score_changed(env, user, current, new_score);

    new_score
}
```

**Best Practice**: Get → Validate → Update → Event

### Delete Pattern

```rust
pub fn remove_score(env: &Env, user: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Score(user.clone()));
}
```

**Best Practice**: Rarely needed, be careful with financial data

## Storage Optimization

### 1. Minimize Storage Operations

```rust
// ❌ Bad: Multiple reads
let score1 = get_score(env, user);
let score2 = get_score(env, user);
let score3 = get_score(env, user);

// ✅ Good: Single read
let score = get_score(env, user);
use_score(score);
use_score(score);
use_score(score);
```

### 2. Batch Operations

```rust
// ❌ Bad: Loop with individual writes
for user in users.iter() {
    set_score(env, user, calculate(user));
}

// ✅ Good: Collect and batch write
let scores: Vec<_> = users.iter()
    .map(|user| (user, calculate(user)))
    .collect();

for (user, score) in scores {
    set_score(env, user, score);
}
```

### 3. Use Appropriate Storage Type

```rust
// ❌ Bad: Persistent for temporary cache
env.storage().persistent().set(&CacheKey, &temp_value);

// ✅ Good: Temporary for cache
env.storage().temporary().set(&CacheKey, &temp_value);
```

### 4. Clean Up Unused Data

```rust
pub fn cleanup_expired_loans(env: &Env, loan_ids: Vec<u64>) {
    for loan_id in loan_ids {
        let loan = get_loan(env, loan_id);
        if loan.is_completed() {
            env.storage().persistent().remove(&DataKey::Loan(loan_id));
        }
    }
}
```

## TTL Management

### Extend TTL for Active Data

```rust
pub fn extend_score_ttl(env: &Env, user: &Address) {
    const EXTEND_TO: u32 = 518_400;    // ~30 days
    const THRESHOLD: u32 = 259_200;     // ~15 days

    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Score(user.clone()), THRESHOLD, EXTEND_TO);
}
```

**Pattern**: Extend when accessed, if below threshold

### Automatic Extension

```rust
pub fn get_score(env: &Env, user: &Address) -> u32 {
    let key = DataKey::Score(user.clone());

    // Extend TTL on read
    env.storage().persistent().extend_ttl(&key, 259_200, 518_400);

    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(DEFAULT_SCORE)
}
```

## Best Practices

### 1. Choose Storage Type Wisely

- **Instance**: Config, admin, contract-level state
- **Persistent**: Financial data, user balances, loans
- **Temporary**: Caches, non-critical computed values

### 2. Handle Missing Keys

```rust
// ✅ Good: Provide default
let score = env.storage().persistent()
    .get(&key)
    .unwrap_or(DEFAULT_SCORE);

// ✅ Good: Handle explicitly
let score = env.storage().persistent()
    .get(&key)
    .unwrap_or_else(|| {
        // Initialize new user
        initialize_user(env, user);
        DEFAULT_SCORE
    });
```

### 3. Validate Before Write

```rust
pub fn set_score(env: &Env, user: &Address, score: u32) {
    // Validate range
    if score > MAX_SCORE {
        panic_with_error!(env, Error::OutOfBounds);
    }

    // Write
    env.storage().persistent().set(&DataKey::Score(user.clone()), &score);
}
```

### 4. Emit Events for Changes

```rust
pub fn update_score(env: &Env, user: &Address, new_score: u32) {
    let old_score = get_score(env, user);

    set_score(env, user, new_score);

    // Emit event
    env.events().publish(
        (symbol_short!("SCORECHG"), user),
        (old_score, new_score)
    );
}
```

### 5. Document Storage Keys

```rust
/// Storage keys for Reputation contract
pub enum DataKey {
    /// Admin address (instance storage)
    Admin,

    /// User reputation score (persistent)
    /// Maps Address → u32 (0-100)
    Score(Address),

    /// Authorized updater status (persistent)
    /// Maps Address → bool
    Updater(Address),
}
```

## Common Pitfalls

### ❌ Forgetting to Extend TTL

```rust
// Data may expire!
let score = env.storage().persistent().get(&key).unwrap();
```

### ✅ Always Extend for Critical Data

```rust
let key = DataKey::Score(user.clone());
env.storage().persistent().extend_ttl(&key, 259_200, 518_400);
let score = env.storage().persistent().get(&key).unwrap();
```

### ❌ Using Wrong Storage Type

```rust
// Admin changes rarely, don't use persistent
env.storage().persistent().set(&DataKey::Admin, &admin);
```

### ✅ Use Instance for Config

```rust
// Admin is config-level, use instance
env.storage().instance().set(&DataKey::Admin, &admin);
```

### ❌ Not Handling Missing Keys

```rust
// Panics if user not found!
let score: u32 = env.storage().persistent().get(&key).unwrap();
```

### ✅ Provide Defaults

```rust
let score: u32 = env.storage().persistent()
    .get(&key)
    .unwrap_or(DEFAULT_SCORE);
```

## Resources

- [Soroban Storage Docs](https://soroban.stellar.org/docs/learn/storage)
- [Storage Types Guide](https://soroban.stellar.org/docs/learn/storage#storage-types)
- [TTL Management](https://soroban.stellar.org/docs/learn/storage#ttl-management)
