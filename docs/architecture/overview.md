# Architecture

## Technology Stack
- **Blockchain**: Stellar
- **Platform**: Soroban (WASM smart contracts)
- **Language**: Rust
- **SDK**: soroban-sdk 21.7.1
- **Build**: Cargo + wasm32-unknown-unknown target

## Contract Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ Reputation  │◄────┤  CreditLine  │────►│   Merchant   │
│             │     │              │     │   Registry   │
└─────────────┘     └──────────────┘     └──────────────┘
      ▲                     │
      │                     ▼
      │             ┌──────────────┐
      └─────────────┤  Liquidity   │
                    │     Pool     │
                    └──────────────┘
```

### Data Flows

**Loan Creation**:
User → CreditLine → Merchant Registry (validate) → Reputation (check score) → Liquidity Pool (fund) → Merchant

**Repayment**:
User → CreditLine → Liquidity Pool (return funds) → Reputation (increase score)

**Default**:
CreditLine → Reputation (decrease score) → Liquidity Pool (receive guarantee)

## Module Pattern

All contracts follow this structure:

```
contracts/<contract-name>/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        # Contract entry point, public functions
    ├── types.rs      # Constants, type definitions
    ├── errors.rs     # Error enums
    ├── storage.rs    # Storage operations
    ├── access.rs     # Authorization checks
    ├── events.rs     # Event emission
    └── tests.rs      # Test suite
```

### Module Responsibilities

| Module | Purpose | Example |
|--------|---------|---------|
| `lib.rs` | Public API | `#[contractimpl]` functions |
| `types.rs` | Constants | `const MAX_SCORE: u32 = 100` |
| `errors.rs` | Error enums | `#[contracterror] enum` |
| `storage.rs` | State mgmt | `get_score()`, `set_score()` |
| `access.rs` | Auth checks | `require_admin()`, `require_updater()` |
| `events.rs` | Event helpers | `emit_score_changed()` |
| `tests.rs` | Tests | `#[test] fn it_works()` |

## Storage

**Soroban Map-based storage**:
```rust
pub enum DataKey {
    Admin,                // Single value
    Updater(Address),     // Map: Address → bool
    Score(Address),       // Map: Address → u32
}

// Usage
env.storage().instance().get(&key).unwrap_or(default)
env.storage().instance().set(&key, &value)
```

## Authorization

**Soroban native auth**:
```rust
address.require_auth();  // Verify signature

// Role-based
fn require_admin(env: &Env, admin: &Address) {
    if get_admin(env) != *admin {
        panic_with_error!(env, Error::NotAdmin);
    }
    admin.require_auth();
}
```

## Events

**Structure**:
```rust
env.events().publish(
    (symbol_short!("TOPIC"), indexed_address),  // Topics (indexed)
    (data1, data2, data3)                        // Data (non-indexed)
);
```

**Reputation Contract Events**:
- `SCORECHGD`: Score changed (user, old, new, reason)
- `UPDCHGD`: Updater status changed (updater, allowed)
- `ADMINCHGD`: Admin changed (old_admin, new_admin)

## Error Handling

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    ErrorName = 1,  // Sequential codes starting from 1
}

// Usage
if invalid { panic_with_error!(&env, Error::ErrorName); }

// Safe arithmetic
let result = value.checked_add(amount).ok_or(Error::Overflow)?;
```

## Build Configuration

**Workspace Cargo.toml**:
```toml
[workspace]
members = ["contracts/reputation-contract"]
resolver = "2"

[profile.release]
opt-level = "z"        # Size optimization
overflow-checks = true
lto = true
strip = "symbols"
```

**Contract Cargo.toml**:
```toml
[lib]
crate-type = ["cdylib"]  # Required for WASM

[dependencies]
soroban-sdk = "21.7.1"

[dev-dependencies]
soroban-sdk = { version = "21.7.1", features = ["testutils"] }
```

## Build Commands

```bash
# Check
cargo check

# Test
cargo test

# Build native
cargo build --release

# Build WASM
cargo build -p <contract> --target wasm32-unknown-unknown --release
# Output: target/wasm32-unknown-unknown/release/<contract>.wasm
```

## Testing Pattern

```rust
#[test]
fn test_name() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let addr = Address::generate(&env);

    // Execute & verify
    client.function(&addr);
    assert_eq!(client.get_value(&addr), expected);
}
```

## Security Principles

1. **Check auth first**: Always verify authorization before state changes
2. **Validate inputs**: Range checks, address validation
3. **Safe math**: Use `checked_add/sub/mul/div`
4. **Emit events**: Log all state changes
5. **Fail securely**: Panic on unexpected conditions

## Performance

- **WASM size**: Target <64KB per contract
- **Storage**: Minimize reads/writes
- **Gas optimization**: Batch operations when possible
