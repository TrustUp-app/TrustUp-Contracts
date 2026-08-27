# Liquidity Pool Contract

## Purpose

The liquidity pool holds the capital that funds TrustUp loans. Liquidity providers
(LPs) `deposit` a SEP-41 token (e.g. USDC) and receive **shares** representing a
proportional claim on the pool; the CreditLine contract draws on that capital via
`fund_loan` and returns principal plus interest via `receive_repayment`. Interest
is split 85% to LPs (kept in the pool, which raises the share price), 10% to the
protocol treasury and 5% to the merchant incentive fund.

Shares are a **fungible, pool-wide claim**: one share is worth
`total_liquidity / total_shares` tokens, and is never tied to a specific loan.

## Utilization-based interest rate model

The pool computes a **dynamic borrow rate** from its own utilization using an
Aave-style *kinked* curve. Utilization is `locked_liquidity / total_liquidity`
and every value is basis points (10000 = 100%). The curve has a kink at
`optimal_utilization`: below it the rate rises gently with `slope1`, above it
steeply with `slope2`, which pushes back on borrowing as the pool drains.

```text
u <= optimal:  rate = base + slope1 * u
u >  optimal:  rate = base + slope1 * optimal + slope2 * (u - optimal)
```
(`u` and `optimal` taken as fractions, i.e. bps / 10000)

Parameters are seeded at `initialize` with sensible defaults (base 2%, slope1
+4% to the kink, slope2 +60% after, optimal 80%) and are retunable by governance
via `set_rate_params` (admin-gated, validated: `0 < optimal < 10000`, non-negative
base/slopes).

`quote_interest(principal, reputation_discount_bps)` is the wiring point for
`creditline-contract`: it applies the current utilization rate to `principal`,
then subtracts CreditLine's reputation-based discount, floored at `base_rate` so
a discount can never price a loan below the pool's base cost. This replaces the
legacy fixed `base_interest_bps` path (previously the pool never computed a rate).

## Transferable LP shares (secondary market)

LP positions are transferable. Instead of being forced to `withdraw` — which is
capped by the pool's *available* liquidity and removes capital from the protocol —
a provider can sell or move their position to another address with SEP-41-style
`approve` / `transfer` / `transfer_from` semantics.

```rust
// Owner grants an operator/buyer the right to pull shares, until `expiration_ledger`
pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32)
pub fn allowance(env: Env, from: Address, spender: Address) -> i128

// Direct transfer, signed by the share owner
pub fn transfer(env: Env, from: Address, to: Address, amount: i128)

// Delegated transfer, signed by the spender and paid out of their allowance
pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128)
```

**Events**

| Symbol | Emitted by | Topics | Data |
|--------|-----------|--------|------|
| `LQAPPRV` | `approve` | `(symbol, from, spender)` | `(amount, expiration_ledger)` |
| `LQXFER` | `transfer`, `transfer_from` | `(symbol, from, to)` | `amount` |

`LQXFER` always names the share owner and the recipient — never the spender — so
indexers see the same shape for both transfer paths.

### Semantics

- **Allowances expire.** `approve` stores `(amount, expiration_ledger)` in
  temporary storage whose TTL tracks the expiration. An allowance is spendable
  while `expiration_ledger >= current ledger sequence` (inclusive); afterwards it
  reads as `0` and cannot be spent. `amount == 0` revokes, and is accepted with any
  expiration ledger so revocation can never be blocked.
- **Approve overwrites**, it does not accumulate. Approving more than the current
  balance is allowed; the balance is checked when the allowance is spent.
- **Authorization**: `transfer` requires the owner's auth; `transfer_from` requires
  only the spender's auth (the owner already signed at `approve` time).
- **Self-transfers** are accepted and are a strict no-op on balances — a share can
  never be minted by sending it to yourself. `transfer_from` still spends the
  allowance in that case.
- **Amounts must be positive.** Zero-value transfers are rejected
  (`InvalidAmount`), consistent with `deposit`/`withdraw` in this contract.
- **Pausing** blocks `transfer` and `transfer_from`, like every other value
  movement. `approve` and `allowance` stay available while paused, since neither
  moves value.
- Transfers run under the same reentrancy guard as deposits and withdrawals, so a
  hostile token cannot move shares from inside a token callback.

### Why transfers are not restricted by loan exposure

Share transfers are **accounting-neutral**: `total_shares`, `total_liquidity` and
`locked_liquidity` are all unchanged by a transfer — only the owner of the claim
changes. All transfer paths funnel through one internal `move_shares` helper, which
is where that invariant is enforced (sum of balances preserved, `total_shares`
never touched), so `PoolStats` stays consistent by construction.

Because this pool tracks loan exposure globally (`locked_liquidity`) rather than
per provider, no individual LP's shares "back" a particular loan — every share
backs every loan proportionally. Blocking transfers whenever `locked_liquidity > 0`
would therefore freeze *all* LP positions for as long as any loan is outstanding,
without protecting anything: a transfer moves no tokens.

Exposure is enforced at the only point where value actually leaves the pool.
`withdraw` is capped by `total_liquidity - locked_liquidity`, and that cap applies
to the buyer exactly as it applied to the seller. A buyer who acquires 600 shares
while 800 of a 1,000-token pool is out on loan can withdraw at most the 200
available — buying shares can never unlock loaned-out liquidity.

## Public API

```rust
// Setup
pub fn initialize(env: Env, admin: Address, token: Address, treasury: Address, merchant_fund: Address)

// Admin
pub fn set_creditline(env: Env, admin: Address, creditline: Address)
pub fn set_treasury(env: Env, admin: Address, treasury: Address)
pub fn set_merchant_fund(env: Env, admin: Address, merchant_fund: Address)
pub fn set_admin(env: Env, admin: Address, new_admin: Address)
pub fn pause(env: Env, admin: Address)
pub fn unpause(env: Env, admin: Address)
pub fn set_rate_params(env: Env, admin: Address, base_rate_bps: i128, slope1_bps: i128, slope2_bps: i128, optimal_utilization_bps: i128)

// Interest rate model
pub fn get_rate_params(env: Env) -> RateParams
pub fn get_utilization_bps(env: Env) -> i128            // current utilization (bps)
pub fn get_current_rate_bps(env: Env) -> i128           // rate at current utilization (bps)
pub fn quote_rate_bps(env: Env, utilization_bps: i128) -> i128  // rate at a hypothetical utilization
pub fn quote_interest(env: Env, principal: i128, reputation_discount_bps: i128) -> i128

// LP operations
pub fn deposit(env: Env, provider: Address, amount: i128) -> i128   // returns shares issued
pub fn withdraw(env: Env, provider: Address, shares: i128) -> i128  // returns tokens returned

// Share transfers (secondary market)
pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32)
pub fn allowance(env: Env, from: Address, spender: Address) -> i128
pub fn transfer(env: Env, from: Address, to: Address, amount: i128)
pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128)

// CreditLine operations (restricted)
pub fn fund_loan(env: Env, creditline: Address, merchant: Address, amount: i128)
pub fn receive_repayment(env: Env, creditline: Address, principal: i128, interest: i128, unlock: i128)
pub fn receive_guarantee(env: Env, creditline: Address, amount: i128)
pub fn distribute_interest(env: Env, caller: Address, interest_amount: i128)

// Queries
pub fn get_pool_stats(env: Env) -> PoolStats
pub fn get_lp_shares(env: Env, provider: Address) -> i128
pub fn calculate_withdrawal(env: Env, shares: i128) -> i128
pub fn get_admin(env: Env) -> Address
pub fn get_token(env: Env) -> Address
pub fn get_treasury(env: Env) -> Option<Address>
pub fn get_merchant_fund(env: Env) -> Option<Address>
pub fn get_creditline(env: Env) -> Option<Address>
pub fn is_paused(env: Env) -> bool
```

## Storage

| Key | Storage | Value |
|-----|---------|-------|
| `ADMIN`, `TOKEN`, `CRDTLIN`, `TREASURY`, `MRCHFND` | instance | configured addresses |
| `TOTSHRS`, `TOTLIQ`, `LCKDLIQ` | instance | pool totals |
| `LOCKED`, `PAUSED` | instance | reentrancy guard, pause flag |
| `RATEPARM` | instance | `RateParams { base_rate_bps, slope1_bps, slope2_bps, optimal_utilization_bps }` |
| `(LPSHRS, Address)` | persistent | shares owned by a provider |
| `(LPALLOW, AllowanceKey)` | temporary | `AllowanceValue { amount, expiration_ledger }` |

Allowances live in temporary storage because they are self-expiring by design:
the entry's TTL is extended to the approved expiration ledger and nothing needs to
outlive it.

## Errors

| Code | Name | When |
|------|------|------|
| 1 | `NotAdmin` | Caller is not the admin |
| 2 | `AlreadyInitialized` | `initialize` called twice |
| 3 | `NotInitialized` | Contract used before initialization |
| 4 | `InvalidAmount` | Amount ≤ 0 (deposit, withdraw, transfer, negative approval) |
| 5 | `InsufficientShares` | Balance below the shares being withdrawn or transferred |
| 6 | `InsufficientLiquidity` | Requested amount exceeds available (unlocked) liquidity |
| 7 | `Overflow` | Arithmetic overflow |
| 8 | `Underflow` | Arithmetic underflow |
| 9 | `NotCreditLine` | Caller is not the registered CreditLine |
| 10 | `ZeroTotalShares` | Withdrawal while the pool has no shares |
| 11 | `ReentrancyDetected` | Reentrant call blocked |
| 12 | `ContractPaused` | Operation attempted while paused |
| 13 | `InsufficientAllowance` | `transfer_from` exceeds (or has no) live allowance |
| 14 | `InvalidExpirationLedger` | Approval expires in the past or beyond the max entry TTL |
| 15 | `InvalidRateParams` | Rate curve params invalid (`optimal` outside `0 < x < 10000`, or negative base/slope) |

## Testing

```bash
cargo test -p liquidity-pool-contract
```
