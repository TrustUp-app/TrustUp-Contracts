# Merchant Registry Contract

## Purpose

Maintain the on-chain list of merchants that are allowed to receive BNPL payments in the TrustUp protocol. Only the admin can register merchants and change their status; anyone can query them.

## Overview

Each merchant is stored under its address together with a `MerchantInfo` record:

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Merchant display name (1-64 characters) |
| `registration_date` | `u64` | Ledger timestamp at registration |
| `active` | `bool` | Whether the merchant is currently allowed to operate |
| `total_sales` | `u64` | Starts at `0` on registration |

The contract also keeps a running count of registered merchants.

## Available Functions

### Initialization
- `initialize(admin: Address)` - Set the admin. Can only be called once (`AlreadyInitialized` otherwise)

### Admin Operations (require admin auth)
- `register_merchant(admin: Address, merchant: Address, name: String)` - Register a new active merchant. Fails with `MerchantAlreadyRegistered` for duplicates and `InvalidName` if the name is empty or longer than 64 characters
- `deactivate_merchant(admin: Address, merchant: Address)` - Mark a registered merchant as inactive
- `activate_merchant(admin: Address, merchant: Address)` - Mark a registered merchant as active
- `set_merchant_status(admin: Address, merchant: Address, active: bool)` - Set the active flag directly

### Queries
- `is_active(merchant: Address) -> bool` - Returns `true` only if the merchant is registered and active
- `get_merchant_info(merchant: Address) -> MerchantInfo` - Get the merchant record (`MerchantNotFound` if unknown)
- `get_merchant_count() -> u64` - Total number of registered merchants

## Events

| Event topic | Topics | Data | Emitted by |
|-------------|--------|------|------------|
| `MERCHTREG` | `merchant` | `name: String` | `register_merchant` |
| `MERCHTSTATUS` | `merchant` | `active: bool` | `activate_merchant`, `deactivate_merchant`, `set_merchant_status` |

## Errors

| Error | Code |
|-------|------|
| `AlreadyInitialized` | 1 |
| `NotInitialized` | 2 |
| `MerchantAlreadyRegistered` | 3 |
| `MerchantNotFound` | 4 |
| `InvalidName` | 5 |
| `Unauthorized` | 6 |

## Build Instructions

### Prerequisites

Install the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
```

### Build

```bash
cargo build -p merchant-registry-contract --target wasm32-unknown-unknown --release
```

The compiled WebAssembly binary will be located at:
```
target/wasm32-unknown-unknown/release/merchant_registry_contract.wasm
```

### Test

```bash
cargo test -p merchant-registry-contract
```

## Architecture

- `lib.rs` - Contract entry points
- `types.rs` - `DataKey` and `MerchantInfo` definitions
- `storage.rs` - Admin, merchant and counter storage
- `access.rs` - Admin authorization check
- `events.rs` - Event emission
- `errors.rs` - Error type definitions
- `tests.rs` - Test suite

## Authorization

All admin operations call `require_auth()` on the supplied `admin` address and then check it against the stored admin; any other address gets `Unauthorized`.
