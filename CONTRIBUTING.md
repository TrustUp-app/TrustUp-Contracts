# Contributing Guide

## Setup

```bash
# Clone
git clone https://github.com/yourusername/TrustUp-Contracts.git
cd TrustUp-Contracts

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify
cargo check && cargo test
```

## Branch Naming

```
feat/<issue-id>-<description>   # feat/SC-08-loan-creation
fix/<description>                # fix/reputation-overflow
docs/<description>               # docs/update-architecture
test/<description>               # test/creditline-boundary-tests
refactor/<description>           # refactor/extract-storage
ci/<description>                 # ci/add-coverage
```

## Commit Standards

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: implement loan creation (SC-08)
fix: prevent score overflow in increase_score
docs: update roadmap with completed issues
test: add boundary tests for reputation
refactor: extract storage to separate module
ci: add code coverage reporting
```

**Atomic Commits**: One logical change per commit.

## Code Standards

### Formatting
```bash
cargo fmt           # Format code
cargo clippy        # Lint code
```

### Module Pattern
```
src/
├── lib.rs         # Contract public API
├── types.rs       # Constants, type defs
├── errors.rs      # Error enums
├── storage.rs     # Storage operations
├── access.rs      # Authorization
├── events.rs      # Event helpers
└── tests.rs       # Tests
```

### Error Handling
```rust
// Define errors
#[contracterror]
pub enum MyError {
    InvalidInput = 1,
    Unauthorized = 2,
}

// Use checked arithmetic
let result = value.checked_add(amount).ok_or(MyError::Overflow)?;

// Panic with error
if invalid { panic_with_error!(&env, MyError::InvalidInput); }
```

### Authorization
```rust
// Check auth first
pub fn admin_function(env: Env, admin: Address) {
    require_admin(&env, &admin);  // ← Check first
    // ... state changes
}
```

### Testing
```rust
#[test]
fn test_name() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Execute & verify
    client.function(&param);
    assert_eq!(client.getter(), expected);
}

#[test]
#[should_panic(expected = "ErrorName")]
fn test_error() {
    // Test error conditions
}
```

## Workflow

1. Pick issue from [Roadmap](ROADMAP.md)
2. Create branch: `git checkout -b feat/SC-XX-description`
3. Make changes (follow code standards)
4. Write tests (>90% coverage goal)
5. Run checks:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   cargo build --target wasm32-unknown-unknown --release
   ```
6. Commit atomically
7. Push & create PR
8. Address review feedback

## Pull Request

**Title**: `<type>: <description> (<issue-id>)`

**Template**:
```markdown
## Description
Brief description of changes.

## Related Issue
Closes #XX or Implements SC-XX

## Type
- [ ] Bug fix
- [x] New feature
- [ ] Breaking change
- [ ] Documentation

## Testing
- [x] Tests pass
- [x] New tests added
- [x] WASM builds

## Checklist
- [x] Code formatted (cargo fmt)
- [x] Linting passed (cargo clippy)
- [x] Tests written
- [x] Documentation updated
```

## Common Commands

```bash
# Build
cargo build --release
cargo build -p <contract> --target wasm32-unknown-unknown --release

# Test
cargo test
cargo test -p <contract>
cargo test <test_name>

# Check
cargo check
cargo clippy -- -D warnings

# Format
cargo fmt
cargo fmt -- --check
```

## Adding New Contract

1. Create directory: `contracts/my-contract/src`
2. Create `Cargo.toml`:
   ```toml
   [package]
   name = "my-contract"
   version = "0.1.0"
   edition = "2021"

   [lib]
   crate-type = ["cdylib"]

   [dependencies]
   soroban-sdk = "21.7.1"

   [dev-dependencies]
   soroban-sdk = { version = "21.7.1", features = ["testutils"] }
   ```
3. Add to workspace in root `Cargo.toml`:
   ```toml
   [workspace]
   members = ["contracts/reputation-contract", "contracts/my-contract"]
   ```
4. Create modules: `lib.rs`, `types.rs`, `errors.rs`, `storage.rs`, `access.rs`, `events.rs`, `tests.rs`
5. Add README.md

## Resources

- [Soroban Docs](https://soroban.stellar.org/docs)
- [Stellar Dev Docs](https://developers.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Conventional Commits](https://www.conventionalcommits.org/)
