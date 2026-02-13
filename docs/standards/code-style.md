# Code Style Guide

Rust code style and conventions for TrustUp smart contracts.

## Formatting

### Use rustfmt

Always format code before committing:
```bash
cargo fmt
```

### Check Formatting

```bash
cargo fmt -- --check
```

### CI Enforcement

Our CI pipeline enforces formatting. All PRs must pass formatting checks.

## Linting

### Use Clippy

Run clippy to catch common mistakes:
```bash
cargo clippy
```

### Treat Warnings as Errors

```bash
cargo clippy -- -D warnings
```

### Fix Clippy Suggestions

Apply automatic fixes:
```bash
cargo clippy --fix
```

## Code Organization

### Contract Structure

```rust
// 1. Crate attributes
#![no_std]

// 2. Module declarations
mod types;
mod errors;
mod storage;
mod access;
mod events;

#[cfg(test)]
mod tests;

// 3. Imports
use soroban_sdk::{contract, contractimpl, Address, Env};
use types::*;
use errors::*;

// 4. Contract definition
#[contract]
pub struct MyContract;

// 5. Implementation
#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env, param: Address) -> u32 {
        // Implementation
    }
}
```

### Module Organization

**lib.rs**: Public API only
```rust
#[contractimpl]
impl ReputationContract {
    pub fn get_score(env: Env, user: Address) -> u32 {
        storage::get_score(&env, &user)
    }

    pub fn increase_score(env: Env, admin: Address, user: Address, amount: u32) {
        access::require_updater(&env, &admin);

        let old_score = storage::get_score(&env, &user);
        let new_score = old_score.checked_add(amount).unwrap();

        storage::set_score(&env, &user, new_score);
        events::emit_score_changed(&env, &user, old_score, new_score);
    }
}
```

**storage.rs**: Pure storage operations
```rust
pub fn get_score(env: &Env, user: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Score(user.clone()))
        .unwrap_or(DEFAULT_SCORE)
}

pub fn set_score(env: &Env, user: &Address, score: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::Score(user.clone()), &score);
}
```

**access.rs**: Authorization checks
```rust
pub fn require_admin(env: &Env) -> Address {
    let admin = storage::get_admin(env);
    admin.require_auth();
    admin
}

pub fn require_updater(env: &Env, updater: &Address) {
    if !storage::is_updater(env, updater) {
        panic_with_error!(env, Error::NotUpdater);
    }
    updater.require_auth();
}
```

## Naming Conventions

### Functions

```rust
// ✅ Good: Descriptive, action-oriented
pub fn get_score(env: Env, user: Address) -> u32
pub fn increase_score(env: Env, user: Address, amount: u32)
pub fn set_admin(env: Env, new_admin: Address)

// ❌ Bad: Unclear, abbreviated
pub fn gs(env: Env, u: Address) -> u32
pub fn inc(env: Env, u: Address, a: u32)
pub fn admin(env: Env, a: Address)
```

### Types

```rust
// ✅ Good: PascalCase, descriptive
pub struct LoanInfo {
    borrower: Address,
    amount: i128,
    due_date: u64,
}

pub enum DataKey {
    Score(Address),
    Loan(u64),
}

// ❌ Bad: snake_case, unclear
pub struct loan_info {
    b: Address,
    amt: i128,
    due: u64,
}
```

### Constants

```rust
// ✅ Good: SCREAMING_SNAKE_CASE, descriptive
const MAX_SCORE: u32 = 100;
const DEFAULT_SCORE: u32 = 50;
const MIN_GUARANTEE_PERCENT: u32 = 20;

// ❌ Bad: lowercase, unclear
const max: u32 = 100;
const def: u32 = 50;
const min_g: u32 = 20;
```

### Variables

```rust
// ✅ Good: snake_case, descriptive
let user_score = get_score(&env, &user);
let new_balance = old_balance + amount;
let is_admin = check_admin(&env, &address);

// ❌ Bad: camelCase, single letter
let userScore = get_score(&env, &user);
let nb = old_balance + amount;
let x = check_admin(&env, &address);
```

## Comments

### When to Comment

**Do comment**:
- Complex algorithms
- Non-obvious business logic
- Public API functions
- Safety invariants
- TODOs with issue numbers

**Don't comment**:
- Obvious code
- What the code does (the code shows that)
- Redundant information

### Doc Comments

Use `///` for public APIs:
```rust
/// Returns the reputation score for a user.
///
/// Returns DEFAULT_SCORE (50) for users without a score.
///
/// # Arguments
/// * `user` - The address of the user
///
/// # Returns
/// The user's reputation score (0-100)
pub fn get_score(env: Env, user: Address) -> u32 {
    storage::get_score(&env, &user)
}
```

### Inline Comments

Use `//` sparingly for complex logic:
```rust
pub fn calculate_interest(principal: i128, rate: u32, days: u32) -> i128 {
    // Interest = P * R * T / 365 / 100
    // Using checked operations to prevent overflow
    let daily_rate = principal
        .checked_mul(rate as i128)
        .unwrap()
        .checked_mul(days as i128)
        .unwrap();

    daily_rate / 365 / 100
}
```

### TODO Comments

Always include issue number:
```rust
// TODO(SC-15): Implement automatic interest distribution
// TODO(SC-20): Add comprehensive integration tests
```

## Error Handling

### Define Errors Clearly

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReputationError {
    NotAdmin = 1,
    NotUpdater = 2,
    OutOfBounds = 3,
    Overflow = 4,
    Underflow = 5,
}
```

### Use Checked Operations

```rust
// ✅ Good: Checked operations
let result = value.checked_add(amount).ok_or(Error::Overflow)?;
let difference = value.checked_sub(amount).ok_or(Error::Underflow)?;

// ❌ Bad: Unchecked operations (can panic unexpectedly)
let result = value + amount;
let difference = value - amount;
```

### Fail Fast

```rust
// ✅ Good: Validate early
pub fn increase_score(env: Env, updater: Address, user: Address, amount: u32) {
    // Check authorization first
    require_updater(&env, &updater);

    // Validate inputs
    let current = get_score(&env, &user);
    let new_score = current.checked_add(amount).ok_or(Error::Overflow)?;

    if new_score > MAX_SCORE {
        panic_with_error!(&env, Error::OutOfBounds);
    }

    // Proceed with state changes
    set_score(&env, &user, new_score);
}
```

## Testing Style

### Test Function Names

```rust
// ✅ Good: Descriptive test names
#[test]
fn test_increase_score_success() { }

#[test]
fn test_increase_score_overflow() { }

#[test]
fn test_set_admin_unauthorized() { }

// ❌ Bad: Unclear names
#[test]
fn test1() { }

#[test]
fn test_score() { }
```

### Test Structure

```rust
#[test]
fn test_increase_score_success() {
    // Arrange
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    client.set_updater(&admin, &admin, &true);

    // Act
    client.increase_score(&admin, &user, &10);

    // Assert
    assert_eq!(client.get_score(&user), 60);  // 50 + 10
}
```

### Test Error Cases

```rust
#[test]
#[should_panic(expected = "OutOfBounds")]
fn test_increase_score_exceeds_max() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    client.set_updater(&admin, &admin, &true);

    // This should panic: 50 + 60 = 110 > MAX_SCORE (100)
    client.increase_score(&admin, &user, &60);
}
```

## Performance

### Minimize Storage Operations

```rust
// ✅ Good: Single read
pub fn process_loan(env: &Env, loan_id: u64) {
    let loan = storage::get_loan(env, loan_id);
    // Use loan multiple times
    validate_loan(&loan);
    calculate_interest(&loan);
    update_status(&loan);
}

// ❌ Bad: Multiple reads
pub fn process_loan(env: &Env, loan_id: u64) {
    let loan1 = storage::get_loan(env, loan_id);
    validate_loan(&loan1);

    let loan2 = storage::get_loan(env, loan_id);
    calculate_interest(&loan2);

    let loan3 = storage::get_loan(env, loan_id);
    update_status(&loan3);
}
```

### Avoid Unnecessary Cloning

```rust
// ✅ Good: Pass by reference
pub fn validate_address(env: &Env, address: &Address) -> bool {
    storage::check_address(env, address)
}

// ❌ Bad: Unnecessary clone
pub fn validate_address(env: &Env, address: Address) -> bool {
    storage::check_address(env, &address.clone())
}
```

## Security

### Check Authorization First

```rust
// ✅ Good: Auth before state changes
pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
    require_admin(&env, &admin);  // ← Check first
    storage::set_admin(&env, &new_admin);
    events::emit_admin_changed(&env, &admin, &new_admin);
}

// ❌ Bad: State changes before auth
pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
    storage::set_admin(&env, &new_admin);
    require_admin(&env, &admin);  // ← Too late!
}
```

### Validate All Inputs

```rust
// ✅ Good: Comprehensive validation
pub fn create_loan(env: Env, amount: i128, due_date: u64) {
    if amount <= 0 {
        panic_with_error!(&env, Error::InvalidAmount);
    }
    if due_date <= env.ledger().timestamp() {
        panic_with_error!(&env, Error::InvalidDueDate);
    }
    // Proceed...
}
```

## Best Practices Summary

1. **Format**: Always run `cargo fmt`
2. **Lint**: Always run `cargo clippy`
3. **Test**: Write comprehensive tests
4. **Document**: Doc comments for public APIs
5. **Validate**: Check inputs and authorization
6. **Safe Math**: Use checked operations
7. **Events**: Emit for all state changes
8. **Organize**: Follow module patterns
9. **Name**: Descriptive, consistent names
10. **Review**: Self-review before PR

## Tools

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint
cargo clippy

# Lint strictly
cargo clippy -- -D warnings

# Fix lint issues
cargo clippy --fix

# Build
cargo build

# Test
cargo test

# Build WASM
cargo build --target wasm32-unknown-unknown --release
```

## Resources

- [Rust Style Guide](https://doc.rust-lang.org/beta/style-guide/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Soroban Best Practices](https://soroban.stellar.org/docs/best-practices)
