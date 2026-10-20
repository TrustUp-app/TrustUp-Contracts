# Parameters Contract

## Purpose

Store the protocol-wide parameters used by the TrustUp BNPL system (guarantee, reputation, penalties, interest, grace period) and govern how they change, including the protocol pause flag.

## Overview

The contract starts with a single admin and can be migrated once to an M-of-N multi-sig governance model with a timelock. After migration, the legacy admin update path is disabled and every change must go through a proposal.

### Stored parameters (`ProtocolParameters`)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `min_guarantee_percent` | `i128` | 20 | Minimum guarantee percentage (must be 1-100) |
| `min_reputation_threshold` | `u32` | 50 | Minimum reputation score required to borrow |
| `full_repayment_reward` | `u32` | 10 | Reputation reward for fully repaying a loan |
| `default_penalty` | `u32` | 20 | Reputation penalty on default |
| `large_loan_threshold` | `i128` | 5000 | Amount from which a loan is considered large (must be > 0) |
| `large_loan_default_penalty` | `u32` | 30 | Reputation penalty on default of a large loan |
| `base_interest_bps` | `u32` | 0 | Base interest rate in basis points |
| `grace_period_seconds` | `u64` | 0 | Seconds after the last installment due date before a hard default can be triggered (0 disables it) |

The contract also stores the paused flag, the signer set, the approval threshold and the timelock.

## Who can change what

| Action | Who | Condition |
|--------|-----|-----------|
| `initialize` / `initialize_defaults` | The admin being set (requires its auth) | Once |
| `update_parameters`, `set_admin` | Admin | Only before `migrate_to_multisig` (`GovernanceActive` afterwards) |
| `migrate_to_multisig` | Admin | Once |
| `propose_parameters`, `propose_pause`, `approve_proposal` | A signer | After migration |
| `execute_proposal` | Anyone | Approvals reached the threshold; parameter changes also need the timelock to elapse, pause changes execute immediately |
| `cancel_proposal` | The proposer or the admin | Proposal still pending |

## Available Functions

### Initialization
- `initialize(admin: Address, params: ProtocolParameters)` - Set the admin and initial parameters. Can only be called once
- `initialize_defaults(admin: Address)` - Same as `initialize` using the default parameters

### Admin and Parameters
- `get_admin() -> Address` - Get the current admin
- `set_admin(new_admin: Address)` - Replace the admin (current admin auth, only before multi-sig migration)
- `get_parameters() -> ProtocolParameters` - Get the current parameters
- `update_parameters(admin: Address, params: ProtocolParameters)` - Replace the parameters directly (admin, only before multi-sig migration)

### Governance
- `migrate_to_multisig(admin: Address, signers: Vec<Address>, threshold: u32, timelock_secs: u64)` - One-time switch to multi-sig governance
- `propose_parameters(proposer: Address, params: ProtocolParameters) -> u64` - Propose a new parameter set; the proposer's approval is recorded automatically
- `propose_pause(proposer: Address, paused: bool) -> u64` - Propose pausing or unpausing the protocol
- `approve_proposal(signer: Address, proposal_id: u64)` - Approve a pending proposal (once per signer)
- `execute_proposal(caller: Address, proposal_id: u64)` - Apply a proposal that has enough approvals
- `cancel_proposal(caller: Address, proposal_id: u64)` - Cancel a pending proposal

### Queries
- `is_paused() -> bool` - Whether the protocol is paused
- `get_proposal(proposal_id: u64) -> Proposal` - Get a proposal
- `get_signers() -> Vec<Address>` - Get the signer set
- `get_threshold() -> u32` - Get the approval threshold
- `get_timelock() -> u64` - Get the timelock in seconds

## Events

| Event topic | Emitted by |
|-------------|------------|
| `PARMUPDT` | `initialize`, `update_parameters`, `execute_proposal` (parameter change) |
| `PARMADMN` | `set_admin` |
| `PRMMIGR` | `migrate_to_multisig` |
| `PRMPROP` | `propose_parameters`, `propose_pause` |
| `PRMAPPR` | `approve_proposal` |
| `PRMEXEC` | `execute_proposal` |
| `PRMCANC` | `cancel_proposal` |
| `PRMPAUS` | `execute_proposal` (pause) |
| `PRMUNPAU` | `execute_proposal` (unpause) |

## Errors

| Error | Code |
|-------|------|
| `AlreadyInitialized` | 1 |
| `NotAdmin` | 2 |
| `InvalidParameters` | 3 |
| `NotSigner` | 4 |
| `AlreadyApproved` | 5 |
| `ProposalNotFound` | 6 |
| `ProposalNotExecutable` | 7 |
| `ProposalAlreadyFinalized` | 8 |
| `InvalidThreshold` | 9 |
| `AlreadyPaused` | 10 |
| `NotPaused` | 11 |
| `GovernanceActive` | 12 |

## Build Instructions

### Prerequisites

Install the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
```

### Build

```bash
cargo build -p parameters-contract --target wasm32-unknown-unknown --release
```

The compiled WebAssembly binary will be located at:
```
target/wasm32-unknown-unknown/release/parameters_contract.wasm
```

### Test

```bash
cargo test -p parameters-contract
```

## Architecture

- `lib.rs` - Contract entry points and governance logic
- `types.rs` - `ProtocolParameters`, `Proposal`, `DataKey` and default values
- `storage.rs` - Instance storage for admin, parameters and governance settings; persistent storage for proposals
- `access.rs` - Admin and signer checks
- `events.rs` - Event emission
- `errors.rs` - Error type definitions
- `tests.rs` - Test suite
