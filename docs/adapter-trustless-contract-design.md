# Adapter Trustless Contract — Design Document

## Purpose

The `adapter-trustless-contract` is a **trustless escrow and payment adapter** within the TrustUp BNPL system. It acts as the neutral on-chain intermediary that:

1. Holds the borrower's 20% guarantee deposit during the loan lifecycle
2. On successful repayment: returns the guarantee to the borrower
3. On default: forwards the forfeited guarantee to the liquidity pool

The remaining 80% purchase funds are forwarded to the merchant by the CreditLine contract at loan creation; this contract only ever holds the guarantee deposit.

This contract removes the need to trust either the CreditLine contract or any off-chain party with the guarantee funds. All fund movements are encoded in smart contract logic and emitted as auditable on-chain events.

## Problem It Solves

Without this contract, the CreditLine contract would directly hold user guarantee funds. This creates a single point of trust: users must trust the CreditLine contract's admin not to misuse or drain the escrow. The adapter-trustless-contract separates escrow logic from credit logic, minimising blast radius if either contract is compromised and making the guarantee mechanism transparent and auditable.

## Design

### Core Concepts

- **Escrow**: Guarantee funds are locked by `lock_guarantee` at loan creation, identified by `loan_id`.
- **Release**: Only the registered `creditline` contract can call `release_guarantee` (on repayment) or `seize_guarantee` (on default).
- **Token**: A single SEP-41 token address (e.g., USDC) is configured at initialisation.

### State

```
Admin              → Address (instance)
CreditLine         → Address (instance)
Token              → Address (instance)
Escrow(loan_id)    → EscrowEntry { borrower, amount, status } (persistent)
```

### Public API

```rust
// Initialisation
fn initialize(env, admin, creditline, token)

// Admin operations
fn set_admin(env, new_admin)
fn set_creditline(env, admin, creditline)

// Escrow lifecycle (called by CreditLine)
fn lock_guarantee(env, creditline, loan_id, borrower, amount)
fn release_guarantee(env, creditline, loan_id)   // repayment → back to borrower
fn seize_guarantee(env, creditline, loan_id, pool) // default → to liquidity pool

// Queries
fn get_escrow(env, loan_id) -> EscrowEntry
fn get_admin(env) -> Address
fn get_creditline(env) -> Address
```

### Events

- `ESCRWLCK`: Guarantee locked (`loan_id`, `borrower`, `amount`)
- `ESCRWRLS`: Guarantee released to borrower (`loan_id`, `borrower`, `amount`)
- `ESCRWSZD`: Guarantee seized to pool (`loan_id`, `pool`, `amount`)

### Access Control

| Caller     | Allowed operations                             |
|------------|------------------------------------------------|
| Admin      | `set_admin`, `set_creditline`                  |
| CreditLine | `lock_guarantee`, `release_guarantee`, `seize_guarantee` |
| Anyone     | `get_escrow`, `get_admin`, `get_creditline`    |

### Escrow Status Machine

```
(none) ──lock_guarantee──→ Locked
Locked ──release_guarantee──→ Released
Locked ──seize_guarantee──→ Seized
```

Attempting any invalid transition panics with `InvalidStatus`.

## Integration with TrustUp System

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

## Security Considerations

- Only the registered CreditLine address can trigger escrow transitions
- Token address is set once at initialisation (immutable in practice)
- Safe arithmetic on all token amounts
- Status guard prevents double-release or double-seize
- Admin cannot touch escrowed funds directly
