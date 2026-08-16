# Project Context

## What is TrustUp?

TrustUp is a decentralized "Buy Now, Pay Later" (BNPL) platform built on Stellar blockchain using Soroban smart contracts. It enables users to make purchases by paying a 20% guarantee deposit upfront while receiving the remaining 80% as credit from a community-funded liquidity pool. The system uses on-chain reputation to reward good repayment behavior and penalize defaults.

## The Problem

Traditional BNPL and credit systems have several issues:

1. **Centralized control**: Single entities control credit decisions and user data
2. **Opaque scoring**: Users don't understand how creditworthiness is calculated
3. **Limited access**: Many people lack access to traditional credit, especially in emerging markets
4. **High fees**: Centralized platforms charge significant merchant and user fees
5. **Siloed reputation**: Credit history is locked within individual platforms and not portable

## TrustUp's Solution

- **Transparent credit system**: All rules encoded in smart contracts on public blockchain
- **Portable reputation**: On-chain scores owned by users, usable across any dApp
- **Community liquidity**: Decentralized pool of liquidity providers earn interest
- **Lower fees**: No middlemen, automated smart contract execution
- **Financial inclusion**: Accessible to anyone with a Stellar wallet

## How It Works

### User Purchase Flow

1. **Purchase Request**
   - User wants to buy from registered merchant (e.g., $100 laptop)
   - User deposits 20% guarantee ($20)
   - System checks user's reputation score

2. **Loan Creation**
   - Smart contract validates merchant is active
   - Creates loan for 80% ($80) based on user's reputation
   - Determines interest rate and repayment schedule from reputation score
   - Liquidity pool transfers $100 to merchant
   - User's $20 guarantee held in escrow

3. **Repayment**
   - User makes scheduled payments (e.g., 4 monthly payments of $22 = $88 total)
   - Each payment recorded on-chain
   - On-time payments increase reputation
   - Late/missed payments decrease reputation

4. **Completion**
   - **Success**: User repays fully → guarantee returned, reputation +10-15 points
   - **Default**: User misses payments → guarantee forfeited to pool, reputation -20-30 points

### Example: Maria's Laptop Purchase

**Initial State**: Maria has reputation score 75/100

**Transaction**:
- Laptop costs $500
- Maria deposits $100 (20% guarantee)
- TrustUp creates $400 loan at 8% APR (based on score 75)
- Merchant receives $500 from liquidity pool
- Maria owes $432 over 4 months ($108/month)

**Outcome if Maria pays on time**:
- All 4 payments made successfully
- Guarantee ($100) returned to Maria
- Reputation increases to 85/100
- Next purchase: qualifies for better rate (e.g., 6% APR)

**Outcome if Maria defaults**:
- Only 2 payments made, then stops
- Guarantee ($100) forfeited to liquidity pool
- Reputation decreases to 45/100
- Next attempt: denied or very high rate (12%+ APR)

## Reputation System

### Score Mechanics

**Range**: 0-100 (u32 in contract)
- **Default**: New users start at 50 (neutral)
- **Maximum**: 100 (excellent credit history)
- **Minimum**: 0 (severe defaults)

**Score Changes**:
- On-time payment: +2 to +5 points
- Early payment: +5 to +10 points
- Loan completion: +10 to +15 points
- Late payment (1-7 days): -2 to -5 points
- Missed payment (>7 days): -5 to -10 points
- Default: -20 to -30 points

### Credit Tiers

| Score | Tier | Interest Rate | Max Credit | Status |
|-------|------|---------------|------------|--------|
| 90-100 | Excellent | 4-6% APR | $5,000+ | Approved |
| 75-89 | Good | 6-8% APR | $2,000-5,000 | Approved |
| 60-74 | Fair | 8-10% APR | $1,000-2,000 | Approved |
| 40-59 | Poor | 10-15% APR | $500-1,000 | Conditional |
| 0-39 | Very Poor | 15%+ APR | <$500 or Denied | High Risk |

### Reputation Portability

User reputation is:
- **On-chain**: Stored in Stellar blockchain state
- **Owned by user**: Tied to user's Stellar address
- **Portable**: Any dApp can query TrustUp reputation contract
- **Verifiable**: All changes recorded as blockchain events
- **Transparent**: Users see exactly why score changed

**Use cases for portable reputation**:
- Other DeFi protocols can use TrustUp score for lending decisions
- DAOs can use reputation for governance weight
- Future BNPL platforms can leverage existing reputation
- Credit scoring services can aggregate on-chain behavior

## Liquidity Provider Economics

### How LPs Benefit

1. **Passive Income**
   - Deposit funds to liquidity pool
   - Earn interest from loan repayments
   - Typical APY: 5-12% depending on pool utilization

2. **Risk Mitigation**
   - 20% guarantee deposits reduce default exposure
   - Reputation system filters high-risk borrowers
   - Diversification: funds spread across many small loans
   - Default recovery: forfeited guarantees offset losses

3. **Transparent Returns**
   - All interest rates visible on-chain
   - Automated distribution via smart contracts
   - Real-time pool metrics (utilization, default rate, APY)

### LP Journey Example

**Carlos deposits $10,000**:
- Receives pool shares representing his ownership %
- Pool has $100,000 total → Carlos owns 10%

**Pool Activity** (over 6 months):
- Pool funds 200 loans totaling $80,000
- Average interest: 8% APR
- 5% default rate (defaults covered by forfeited guarantees)

**Carlos's Returns**:
- Interest earned: $10,000 × 8% × 0.5 years = $400
- His share: $400 × 10% = $40
- Net APY: ~8% (after defaults)
- Can withdraw anytime (subject to available liquidity)

### Share Mechanics

**Initial Deposit** (first LP):
```
shares_issued = deposit_amount
share_value = 1:1
```

**Subsequent Deposits**:
```
shares_issued = (deposit_amount × total_shares) / total_pool_value
```

**Withdrawal**:
```
withdrawal_amount = (shares_to_burn × total_pool_value) / total_shares
```

**Share Value Increases** when:
- Loans repaid with interest
- Interest accumulates in pool
- Example: Start at 1 share = $1.00 → After 1 year = $1.08

## Economic Model

### Revenue Flows

**Loan Origination**:
- User borrows $80 (80% of $100 purchase)
- Pays $1.60 origination fee (2%)
- Merchant receives $100 from pool
- User owes $81.60 + interest

**Interest Payment** (example: $10 interest payment):
```
$8.50 (85%) → Liquidity Providers
$1.00 (10%) → Protocol Treasury
$0.50 (5%)  → Merchant Incentive Fund
```

### Fee Structure

| Fee Type | Rate | Recipient | Purpose |
|----------|------|-----------|---------|
| Origination | 1-2% | Protocol | Platform sustainability |
| Interest | 4-15% APR | 85% to LPs, 10% protocol, 5% merchants | Credit cost |
| Late Fee | $5-10 flat | Protocol | Discourage defaults |
| Default Penalty | Guarantee forfeited | Liquidity Pool | Cover losses |

### Risk Management

1. **Guarantee Deposits**: 20% upfront reduces loss on default
   - Default on $80 loan → Pool loses max $80 - $20 = $60
   - With 8% interest, breakeven after 7-8 successful loans

2. **Reputation Gating**: Low scores denied or high rates
   - Score <40 → Denied or 15%+ APR
   - Filters out high-risk borrowers

3. **Diversification**: Pool spreads across many loans
   - 1 default in 20 loans = 5% default rate
   - Covered by interest from other 19 loans

4. **Progressive Limits**: Credit increases with good behavior
   - New user (score 50): $500 max
   - Proven user (score 85): $5,000 max
   - Limits exposure per user

## Smart Contract Architecture

### 1. Reputation Contract ✅ (Implemented)

**Purpose**: Manage user credit scores (0-100)

**Key Functions**:
- `get_score(user: Address) -> u32`
- `increase_score(updater: Address, user: Address, amount: u32)`
- `decrease_score(updater: Address, user: Address, amount: u32)`
- `set_admin(admin: Address)` - Transfer admin
- `set_updater(admin: Address, updater: Address, allowed: bool)` - Authorize score updaters

**Access Control**:
- 1 Admin: Can manage updaters
- N Updaters: Can modify scores (typically CreditLine contract)

**Events**:
- `SCORECHGD`: Score changed (user, old_score, new_score, reason)
- `UPDCHGD`: Updater status changed
- `ADMINCHGD`: Admin changed

### 2. CreditLine Contract ⏳ (Planned)

**Purpose**: Handle loan creation, repayment, defaults

**Key Functions** (planned):
- `create_loan(user, merchant, amount, guarantee, schedule) -> loan_id`
- `repay_loan(loan_id, amount) -> remaining_balance`
- `mark_defaulted(loan_id)`

**Interactions**:
- Queries Reputation for user score → determines rate
- Requests funds from Liquidity Pool
- Updates Reputation based on repayment behavior
- Validates merchant via Merchant Registry

### 3. Merchant Registry Contract ⏳ (Planned)

**Purpose**: Whitelist of authorized merchants

**Key Functions** (planned):
- `register_merchant(admin, merchant, name)`
- `is_active_merchant(merchant) -> bool`
- `deactivate_merchant(admin, merchant)`

**Why Needed**: Prevents fraudulent merchants from receiving funds

### 4. Liquidity Pool Contract ⏳ (Planned)

**Purpose**: Manage LP deposits and loan funding

**Key Functions** (planned):
- `deposit(provider, amount) -> shares_issued`
- `withdraw(provider, shares) -> amount_returned`
- `fund_loan(creditline, amount)` - Called by CreditLine
- `receive_repayment(amount)` - Called by CreditLine
- `distribute_interest()` - Update share values

**Mechanics**:
- LPs receive shares proportional to deposit
- Share value increases as interest accumulates
- Withdrawals limited by available (non-loaned) liquidity

## Technology Stack

- **Blockchain**: Stellar
- **Smart Contracts**: Soroban (Rust → WASM)
- **SDK**: soroban-sdk 21.7.1
- **Build**: Cargo
- **Testing**: Soroban testutils
- **Deployment**: Stellar testnet/mainnet

**Why Stellar?**
- Low transaction costs (~$0.00001)
- Fast finality (3-5 seconds)
- Built for payments and financial apps
- WASM-based contracts (efficient, secure)

## Use Cases

### E-Commerce
User buys electronics, furniture, or clothing online with 20% down, pays rest over time.

### Subscription Services
User pays for annual software/streaming subscriptions in monthly installments.

### Digital Goods
Access to online courses, ebooks, or SaaS tools with deferred payment.

### Cross-Border Purchases
Stellar's global reach enables international BNPL transactions with low fees.

### Gig Economy
Freelancers smooth cash flow by financing business expenses (equipment, software).

### Merchant Benefits
- Receive full payment upfront
- Attract more customers (lower barrier to purchase)
- Automated payment processing
- Low fees compared to traditional BNPL (5% vs 2-3%)

## Development Status

**Current Phase**: Phase 2 Complete (Reputation Contract)
**Next Phase**: Phase 3 (CreditLine Contract)

**Completed** (8/20 issues):
- ✅ Admin management (SC-01)
- ✅ Updater authorization (SC-02)
- ✅ Access control events (SC-03)
- ✅ Reputation storage (SC-04)
- ✅ Get reputation function (SC-05)
- ✅ Increase reputation (SC-06)
- ✅ Decrease reputation (SC-07)
- ✅ Reputation tests (SC-18)

**Pending**:
- ⏳ CreditLine: Loan creation, repayment, default (SC-08 to SC-10)
- ⏳ CreditLine ↔ Reputation integration (SC-11, SC-12)
- ⏳ Merchant Registry (SC-13, SC-14)
- ⏳ Liquidity Pool (SC-15 to SC-17)
- ⏳ Remaining tests (SC-19, SC-20)

See [ROADMAP.md](ROADMAP.md) for detailed issue breakdown.

## Future Vision

**Phase 1** (Current): Core BNPL functionality
- Reputation, CreditLine, Merchant Registry, Liquidity Pool

**Phase 2**: Advanced Features
- Multi-chain expansion (deploy to other Stellar-compatible chains)
- Dynamic interest rates based on pool utilization
- Insurance pool for LP protection
- Reputation delegation (high-score users vouch for others)

**Phase 3**: Ecosystem Growth
- ~~Governance token for protocol parameters~~ — superseded by
  `parameters-contract`'s built-in proposal + multi-sig + timelock governance
  (see [architecture/contracts.md](architecture/contracts.md#parameters-contract-)); no separate token needed
- Merchant analytics dashboard
- Credit score aggregation from multiple DeFi protocols
- Integration with traditional credit bureaus (opt-in)

**Phase 4**: Mass Adoption
- Mobile app (user-friendly interface)
- Merchant SDK for easy integration
- Partnerships with e-commerce platforms
- Fiat on/off ramps

## Key Principles

1. **Transparency**: All logic on-chain, all state changes visible
2. **User Ownership**: Reputation belongs to the user, not the platform
3. **Decentralization**: No single point of control or failure
4. **Security**: Comprehensive testing, audits, safe arithmetic
5. **Fairness**: Automated execution, no discrimination
6. **Inclusivity**: Accessible to anyone with internet and wallet

## Resources

- [Architecture](ARCHITECTURE.md) - Technical design
- [Roadmap](ROADMAP.md) - Development phases and issues
- [Contributing](CONTRIBUTING.md) - How to contribute
- [Error Codes](ERROR_CODES.md) - Error reference
