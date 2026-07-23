# Adapter Trustless Contract

## Purpose

TrustUp's other contracts (`reputation-contract`, `creditline-contract`,
`liquidity-pool-contract`, `merchant-registry-contract`, `parameters-contract`) each
gate sensitive operations (changing an admin, updating a cross-contract address,
tuning risk parameters) behind a single `admin` address. That admin key is a single
point of failure and a single point of trust: whoever holds it can unilaterally
change protocol behavior with no on-chain check.

The adapter-trustless-contract removes that single point of trust. It is a generic
**M-of-N multi-sig + timelock gateway** that other contracts (or their admins) can
point their privileged operations through instead of executing them directly:

1. A signer **proposes** an action: the target contract address, the function to
   call, and its arguments.
2. Other signers **approve** the action.
3. Once at least `threshold` signers have approved **and** the configured timelock
   delay has elapsed since the proposal, anyone can **execute** the action, which
   invokes the target contract via `Env::invoke_contract`.

No single signer — including the adapter's own admin — can execute a privileged
call alone. The action, its approvals, and its execution are all public and
auditable on-chain via events.

## What it does *not* do (out of scope for this issue)

- It does not replace each contract's own `admin` field; contracts must be
  reconfigured to name the adapter's resolved address (or an address it controls)
  as their admin/updater if they want its guarantees enforced. That migration is a
  follow-up once the adapter is reviewed and deployed.
- It does not interpret or validate the semantics of the calls it forwards — it is
  a generic relay, not a per-contract policy engine.

## Public API

```rust
// Setup (once)
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

## Events

| Topic       | Emitted when                       |
|-------------|-------------------------------------|
| `ACTNPROP`  | Action proposed                     |
| `ACTNAPPRV` | Signer approves an action           |
| `ACTNREVK`  | Signer revokes their approval       |
| `ACTNEXEC`  | Action executed                     |
| `ACTNCANC`  | Admin cancels an action             |
| `SIGNERADD` | Signer added                        |
| `SIGNERRM`  | Signer removed                      |
| `THRESHCHG` | Approval threshold changed          |
| `TMLOCKCHG` | Timelock delay changed              |
| `ADMINCHGD` | Admin changed                       |

See `docs/architecture/contracts.md` for the full architecture writeup.
