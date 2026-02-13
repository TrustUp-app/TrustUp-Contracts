# File Organization

## Repository Structure

```
TrustUp-Contracts/
├── .github/
│   ├── workflows/
│   │   └── contracts-ci.yml          # CI: build + test on push/PR
│   └── PULL_REQUEST_TEMPLATE.md
│
├── contracts/
│   ├── reputation-contract/          # ✅ Implemented
│   ├── creditline-contract/          # ⏳ Planned (.gitkeep only)
│   ├── merchant-registry-contract/   # ⏳ Planned (.gitkeep only)
│   └── liquidity-pool-contract/      # ⏳ Planned (.gitkeep only)
│
├── docs/
│   ├── ARCHITECTURE.md               # Technical architecture
│   ├── ROADMAP.md                    # Development roadmap (detailed)
│   ├── CONTRIBUTING.md               # Development workflow
│   ├── PROJECT_CONTEXT.md            # Project vision
│   ├── ERROR_CODES.md                # Error reference
│   └── FILE_ORGANIZATION.md          # This file
│
├── target/                            # Build artifacts (git ignored)
│   ├── debug/
│   ├── release/
│   └── wasm32-unknown-unknown/
│       └── release/*.wasm            # Deployable WASM binaries
│
├── Cargo.toml                         # Workspace config
├── Cargo.lock                         # Locked dependencies
├── .gitignore
└── README.md
```

## Workspace Configuration

### Root Cargo.toml
```toml
[workspace]
members = ["contracts/reputation-contract"]
resolver = "2"

[profile.release]
opt-level = "z"         # Size optimization (WASM)
overflow-checks = true  # Safety in release
debug = 0               # No debug symbols
strip = "symbols"       # Strip symbols
lto = true              # Link-time optimization
```

**Why these settings**:
- `opt-level = "z"`: WASM size matters, optimize for size
- `overflow-checks = true`: Safety even in release
- `lto = true`: Aggressive optimization across crates

## Contract Structure

Every contract follows this pattern:

```
contracts/<contract-name>/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        # Contract entry, #[contract], #[contractimpl]
    ├── types.rs      # Constants, type definitions
    ├── errors.rs     # #[contracterror] enum
    ├── storage.rs    # Storage read/write operations
    ├── access.rs     # Authorization helpers
    ├── events.rs     # Event emission helpers
    └── tests.rs      # #[cfg(test)] mod tests
```

### Module Responsibilities

| Module | Contains | Example |
|--------|----------|---------|
| `lib.rs` | `#[contract]` struct, `#[contractimpl]` block, public API | `pub fn get_score()` |
| `types.rs` | Constants, structs, enums, DataKey definitions | `const MAX_SCORE: u32 = 100` |
| `errors.rs` | `#[contracterror]` error enum | `NotAdmin = 1` |
| `storage.rs` | `get_*()`, `set_*()` storage functions | `fn get_score(env, user)` |
| `access.rs` | `require_*()` authorization functions | `fn require_admin()` |
| `events.rs` | `emit_*()` event helpers | `fn emit_score_changed()` |
| `tests.rs` | Test functions | `#[test] fn it_works()` |

### Contract Cargo.toml Template
```toml
[package]
name = "contract-name"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for WASM

[dependencies]
soroban-sdk = "21.7.1"

[dev-dependencies]
soroban-sdk = { version = "21.7.1", features = ["testutils"] }
```

## Build Artifacts

### Target Directory
```
target/
├── debug/
│   └── lib<contract_name>.rlib       # Debug native build
├── release/
│   └── lib<contract_name>.rlib       # Release native build
└── wasm32-unknown-unknown/
    └── release/
        └── <contract_name>.wasm      # ← Deployable WASM
```

### Build Output Naming
- Contract name in `Cargo.toml`: `reputation-contract`
- WASM output: `reputation_contract.wasm` (hyphens → underscores)

### Build Commands
```bash
# Native builds
cargo build                    # → target/debug/
cargo build --release          # → target/release/

# WASM build (for deployment)
cargo build -p reputation-contract --target wasm32-unknown-unknown --release
# → target/wasm32-unknown-unknown/release/reputation_contract.wasm
```

## Storage Pattern

### DataKey Enum
```rust
pub enum DataKey {
    Admin,                // Singleton: Address
    Updater(Address),     // Map: Address → bool
    Score(Address),       // Map: Address → u32
}
```

### Storage Operations
```rust
// Read
env.storage().instance().get(&key).unwrap_or(default)

// Write
env.storage().instance().set(&key, &value)
```

## Contract Public API Pattern

### lib.rs Structure
```rust
#![no_std]

mod types;
mod errors;
mod storage;
mod access;
mod events;

#[cfg(test)]
mod tests;

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env, param: Address) -> u32 {
        // Implementation
    }
}
```

## Error Pattern

### errors.rs
```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MyError {
    ErrorName1 = 1,
    ErrorName2 = 2,
}
```

### Usage
```rust
use soroban_sdk::panic_with_error;

if invalid {
    panic_with_error!(&env, MyError::ErrorName1);
}

// With Result
let value = operation().ok_or(MyError::ErrorName2)?;
```

## Event Pattern

### events.rs
```rust
use soroban_sdk::{Address, Env, Symbol, symbol_short};

pub fn emit_event_name(env: &Env, user: &Address, data: u32) {
    env.events().publish(
        (symbol_short!("EVTNAME"), user),  // Topics (indexed)
        (data,)                             // Data (non-indexed)
    );
}
```

## Test Pattern

### tests.rs
```rust
#![cfg(test)]

use super::*;
use soroban_sdk::{Env, Address};

#[test]
fn test_function() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MyContract, ());
    let client = MyContractClient::new(&env, &contract_id);

    let addr = Address::generate(&env);

    client.my_function(&addr);
    assert_eq!(client.get_value(&addr), expected);
}

#[test]
#[should_panic(expected = "ErrorName")]
fn test_error() {
    // Code that triggers error
}
```

## CI/CD Configuration

### .github/workflows/contracts-ci.yml
```yaml
name: Contracts CI

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with: {toolchain: stable}
      - run: rustup target add wasm32-unknown-unknown
      - run: cargo build --verbose
      - run: cargo build -p reputation-contract --target wasm32-unknown-unknown --release

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with: {toolchain: stable}
      - run: cargo test --verbose
```

## Git Ignore

### .gitignore
```
/target/           # All build artifacts
**/*.rs.bk         # Rustfmt backups
.DS_Store          # macOS
*.swp *.swo        # Vim
.idea/             # JetBrains IDEs
```

**Note**: `Cargo.lock` is **committed** (not ignored) for reproducible builds.

## Adding New Contract

1. **Create structure**:
   ```bash
   mkdir -p contracts/new-contract/src
   touch contracts/new-contract/{Cargo.toml,README.md}
   touch contracts/new-contract/src/{lib,types,errors,storage,access,events,tests}.rs
   ```

2. **Update workspace** in root `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       "contracts/reputation-contract",
       "contracts/new-contract",
   ]
   ```

3. **Implement modules** following patterns above

4. **Verify**:
   ```bash
   cargo check
   cargo test -p new-contract
   cargo build -p new-contract --target wasm32-unknown-unknown --release
   ```

## File Navigation Quick Reference

| Looking for... | Go to... |
|----------------|----------|
| Contract implementation | `contracts/<name>/src/lib.rs` |
| Error definitions | `contracts/<name>/src/errors.rs` |
| Test suite | `contracts/<name>/src/tests.rs` |
| Build config | `Cargo.toml` (workspace or contract-level) |
| WASM output | `target/wasm32-unknown-unknown/release/*.wasm` |
| CI configuration | `.github/workflows/contracts-ci.yml` |
| Roadmap/Issues | `docs/ROADMAP.md` |
| Error catalog | `docs/ERROR_CODES.md` |

## Naming Conventions

- **Files**: `snake_case.rs` (`lib.rs`, `storage.rs`)
- **Modules**: `snake_case` (directory names)
- **Contracts**: `kebab-case` (`reputation-contract`, `creditline-contract`)
- **Functions**: `snake_case` (`get_score`, `require_admin`)
- **Types**: `PascalCase` (`ReputationError`, `DataKey`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`MAX_SCORE`, `DEFAULT_SCORE`)
