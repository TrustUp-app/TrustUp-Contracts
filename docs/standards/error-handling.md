# Error Codes Reference

## Reputation Contract

**Location**: `contracts/reputation-contract/src/errors.rs`

| Code | Name | Description | When | Fix |
|------|------|-------------|------|-----|
| 1 | `NotAdmin` | Caller is not admin | Calling admin-only functions without admin auth | Use admin address or call `set_admin` first |
| 2 | `NotUpdater` | Caller is not authorized updater | Calling score update functions without updater permission | Register as updater via `set_updater` |
| 3 | `OutOfBounds` | Score outside 0-100 range | `increase_score` result >100, or `set_score` with invalid value | Check current score before increasing, cap at MAX_SCORE |
| 4 | `Overflow` | Arithmetic overflow | Addition would exceed u32::MAX (unlikely with 0-100 range) | Use `checked_add`, validate inputs |
| 5 | `Underflow` | Arithmetic underflow | `decrease_score` amount > current score | Check current score before decreasing, use `saturating_sub` |

**Error Definition**:
```rust
#[contracterror]
#[repr(u32)]
pub enum ReputationError {
    NotAdmin = 1,
    NotUpdater = 2,
    OutOfBounds = 3,
    Overflow = 4,
    Underflow = 5,
}
```

## CreditLine Contract (Planned)

| Code | Name | Description |
|------|------|-------------|
| 1 | `InvalidMerchant` | Merchant not registered/inactive |
| 2 | `InsufficientGuarantee` | Guarantee <20% of total |
| 3 | `InsufficientLiquidity` | Pool lacks funds |
| 4 | `LoanNotFound` | Invalid loan ID |
| 5 | `LoanNotActive` | Loan not in Active status |
| 6 | `UnauthorizedRepayment` | Caller not borrower |
| 7 | `InvalidRepaymentAmount` | Amount ≤0 or >balance |
| 8 | `LoanNotOverdue` | Cannot default before due date |
| 9 | `InvalidLoanStatus` | Invalid operation for current status |
| 10 | `LowReputationScore` | Score too low for credit |

## Merchant Registry (Planned)

| Code | Name | Description |
|------|------|-------------|
| 1 | `NotAdmin` | Unauthorized admin action |
| 2 | `MerchantAlreadyRegistered` | Duplicate merchant address |
| 3 | `MerchantNotFound` | Address not registered |
| 4 | `InvalidMerchantName` | Empty or too long name |
| 5 | `MerchantInactive` | Merchant deactivated |

## Liquidity Pool (Planned)

| Code | Name | Description |
|------|------|-------------|
| 1 | `InvalidDepositAmount` | Amount ≤0 |
| 2 | `InsufficientShares` | Not enough shares to withdraw |
| 3 | `InsufficientLiquidity` | Pool lacks available liquidity |
| 4 | `PoolEmpty` | No liquidity (division by zero) |
| 5 | `InvalidWithdrawalAmount` | Amount ≤0 |
| 6 | `OverflowError` | Arithmetic overflow |
| 7 | `TransferFailed` | Token transfer failed |

## Error Handling Patterns

### Define Errors
```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MyError {
    ErrorName = 1,  // Start from 1, sequential
}
```

### Use Errors
```rust
// Panic with error
if invalid_condition {
    panic_with_error!(&env, MyError::ErrorName);
}

// With Result
let result = operation().ok_or(MyError::ErrorName)?;
```

### Safe Arithmetic
```rust
// Checked operations
let sum = a.checked_add(b).ok_or(Error::Overflow)?;
let diff = a.checked_sub(b).ok_or(Error::Underflow)?;

// Saturating (clips at min/max)
let capped = a.saturating_add(b);
```

### Test Errors
```rust
#[test]
#[should_panic(expected = "ErrorName")]
fn test_error_case() {
    // Code that should panic with specific error
}
```

## Best Practices

1. **Sequential codes**: Start from 1, increment by 1
2. **Descriptive names**: ClearPurpose, not Err1
3. **Check early**: Validate inputs before processing
4. **Safe math**: Always use checked_* operations
5. **Test all errors**: Every error code should have a test
