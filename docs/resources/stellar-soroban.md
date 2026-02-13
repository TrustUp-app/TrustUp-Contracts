# Stellar & Soroban Platform

Reference guide for the Stellar blockchain and Soroban smart contracts platform.

## Overview

**Stellar** is a decentralized, open-source blockchain designed for fast, low-cost payments and asset transfers.

**Soroban** is Stellar's smart contracts platform, enabling developers to build decentralized applications using Rust and WebAssembly (WASM).

## Why Stellar?

- **Low Cost**: ~$0.00001 per transaction
- **Fast**: 3-5 second finality
- **Scalable**: Thousands of transactions per second
- **Built for Payments**: Native asset support, path payments
- **Global**: International reach with low fees

## Why Soroban?

- **Rust + WASM**: Type-safe, efficient, and secure
- **Rich SDK**: Comprehensive soroban-sdk for contract development
- **Testing Tools**: Built-in testing utilities
- **Stellar Integration**: Native access to Stellar's payment features
- **Size Optimized**: WASM binaries optimized for blockchain

## Core Concepts

### Accounts

Stellar accounts identified by public keys (G addresses):
```
GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**Key Pair**:
- Public Key (G...): Account identifier
- Secret Key (S...): Signs transactions (keep private!)

### Assets

**Native Asset**: Lumens (XLM) - Stellar's native currency

**Custom Assets**: Create tokens on Stellar
- Identified by asset code + issuer
- Can represent anything (currencies, commodities, securities)

### Contracts

Smart contracts on Soroban:
- Written in Rust
- Compiled to WASM
- Deployed on-chain
- Identified by contract ID (C addresses)

**Contract Address**:
```
CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

### Networks

**Testnet**: Development and testing
- Free XLM from friendbot
- Reset periodically
- No real value

**Mainnet**: Production network
- Real XLM with value
- Permanent state
- Transaction fees required

## Soroban SDK

### Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI (includes Soroban)
cargo install stellar-cli --locked
```

### Core Imports

```rust
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Symbol, Vec, Map,
    symbol_short, panic_with_error,
};
```

### Contract Structure

```rust
#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env, param: Address) -> u32 {
        // Implementation
    }
}
```

### Storage

**Persistent**: Lives forever (until explicitly removed)
```rust
env.storage().persistent().set(&key, &value);
let value = env.storage().persistent().get(&key);
```

**Temporary**: Expires after ~1 day (configurable)
```rust
env.storage().temporary().set(&key, &value);
```

**Instance**: Contract instance-specific, persists with contract
```rust
env.storage().instance().set(&key, &value);
```

### Authorization

```rust
// Require caller to sign
address.require_auth();

// Batch authorization
address.require_auth_for_args((arg1, arg2));
```

### Events

```rust
env.events().publish(
    (symbol_short!("TOPIC"), indexed_param),  // Topics (indexed)
    (data1, data2)                             // Data (not indexed)
);
```

### Testing

```rust
#[test]
fn test_name() {
    let env = Env::default();
    env.mock_all_auths();  // Mock authorization

    let contract_id = env.register(MyContract, ());
    let client = MyContractClient::new(&env, &contract_id);

    let result = client.my_function(&param);
    assert_eq!(result, expected);
}
```

## Stellar CLI

### Installation

```bash
cargo install stellar-cli --locked
```

### Common Commands

#### Network Management
```bash
# Add testnet network
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Add mainnet
stellar network add mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

#### Identity Management
```bash
# Generate new identity
stellar keys generate alice

# Get public key
stellar keys address alice

# Fund testnet account
stellar keys fund alice --network testnet
```

#### Contract Operations
```bash
# Build contract
stellar contract build

# Optimize WASM
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/my_contract.wasm

# Deploy contract
stellar contract deploy \
  --wasm my_contract.wasm \
  --source alice \
  --network testnet

# Install contract WASM (without deploying)
stellar contract install \
  --wasm my_contract.wasm \
  --source alice \
  --network testnet

# Invoke contract function
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  my_function \
  --param1 value1 \
  --param2 value2
```

#### Inspection
```bash
# Read contract storage
stellar contract read \
  --id <CONTRACT_ID> \
  --network testnet

# Get contract events
stellar events \
  --start-ledger <LEDGER> \
  --count 100 \
  --id <CONTRACT_ID>
```

## Development Workflow

### 1. Setup
```bash
# Create new project
cargo new --lib my-contract
cd my-contract

# Add soroban-sdk
cargo add soroban-sdk
```

### 2. Configure Cargo.toml
```toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "22.0.0"

[dev-dependencies]
soroban-sdk = { version = "22.0.0", features = ["testutils"] }

[profile.release]
opt-level = "z"
overflow-checks = true
lto = true
```

### 3. Develop

Write contract in `src/lib.rs`

### 4. Test
```bash
cargo test
```

### 5. Build
```bash
# Build for WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/my_contract.wasm
```

### 6. Deploy
```bash
# Deploy to testnet
stellar contract deploy \
  --wasm my_contract.optimized.wasm \
  --source alice \
  --network testnet
```

### 7. Interact
```bash
# Call contract function
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  my_function --arg value
```

## TrustUp Configuration

### Networks

TrustUp uses:
- **Development**: Testnet
- **Production**: Mainnet (future)

### Accounts

- Admin account: Manages contract upgrades
- Updater accounts: Authorized to update scores
- User accounts: Regular users

### Contracts

| Contract | Purpose | Status |
|----------|---------|--------|
| Reputation | User credit scores | ✅ Deployed (testnet) |
| CreditLine | Loan management | ⏳ In development |
| Merchant Registry | Merchant whitelist | ⏳ Planned |
| Liquidity Pool | LP management | ⏳ Planned |

## Resources

### Official Documentation
- **Stellar Docs**: https://developers.stellar.org/
- **Soroban Docs**: https://soroban.stellar.org/docs
- **SDK Reference**: https://docs.rs/soroban-sdk/

### Tools
- **Stellar Expert**: https://stellar.expert/ (Block explorer)
- **Stellar Laboratory**: https://laboratory.stellar.org/ (Testing tool)
- **Friendbot**: https://friendbot.stellar.org/ (Testnet XLM faucet)

### Tutorials
- **Soroban Quest**: https://quest.stellar.org/soroban (Interactive learning)
- **Examples**: https://github.com/stellar/soroban-examples
- **Docs Tutorials**: https://soroban.stellar.org/docs/tutorials

### Community
- **Discord**: https://discord.gg/stellar (#soroban channel)
- **Stack Overflow**: Tag `stellar` or `soroban`
- **GitHub Discussions**: https://github.com/stellar/soroban-tools/discussions

## Common Patterns

### Access Control
```rust
pub fn require_admin(env: &Env) -> Address {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    admin
}
```

### Safe Arithmetic
```rust
let result = a.checked_add(b).ok_or(Error::Overflow)?;
```

### Event Emission
```rust
pub fn emit_event(env: &Env, user: &Address, value: u32) {
    env.events().publish(
        (symbol_short!("EVENT"), user),
        (value,)
    );
}
```

### Error Handling
```rust
#[contracterror]
#[repr(u32)]
pub enum Error {
    InvalidInput = 1,
    Unauthorized = 2,
}

// Use
if invalid {
    panic_with_error!(&env, Error::InvalidInput);
}
```

## Performance Tips

1. **Minimize Storage**: Storage operations are expensive
2. **Batch Operations**: Combine multiple ops when possible
3. **Optimize WASM**: Always use `stellar contract optimize`
4. **Use Correct Storage Types**:
   - Persistent for critical data
   - Temporary for caches
5. **Avoid Large Data**: Keep contract state small

## Security Best Practices

1. **Auth First**: Check authorization before state changes
2. **Validate Inputs**: Check ranges, addresses, amounts
3. **Safe Math**: Always use checked operations
4. **Emit Events**: Log all important state changes
5. **Fail Securely**: Panic on unexpected conditions
6. **Test Thoroughly**: Cover all edge cases
7. **Audit**: Get contracts audited before mainnet

## Next Steps

1. Complete [Soroban Quest](https://quest.stellar.org/soroban)
2. Review [Soroban Examples](https://github.com/stellar/soroban-examples)
3. Read [Best Practices](https://soroban.stellar.org/docs/best-practices)
4. Join [Stellar Discord](https://discord.gg/stellar)
5. Build and deploy your first contract
