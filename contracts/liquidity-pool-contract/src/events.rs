use soroban_sdk::{symbol_short, Address, Env, Symbol};

const DEPOSITED: Symbol = symbol_short!("LQDEPST");
const WITHDRAWN: Symbol = symbol_short!("LQWTHDR");
const LOAN_FUNDED: Symbol = symbol_short!("LQFUND");
const REPAYMENT_RCV: Symbol = symbol_short!("LQREPAY");
const GUARANTEE_RCV: Symbol = symbol_short!("LQGUART");
const INTEREST_DIST: Symbol = symbol_short!("LQINTDST");

/// Emitted when a liquidity provider deposits tokens
pub fn emit_liquidity_deposited(
    env: &Env,
    provider: &Address,
    amount: i128,
    shares_issued: i128,
) {
    env.events()
        .publish((DEPOSITED, provider), (amount, shares_issued));
}

/// Emitted when a liquidity provider withdraws tokens
pub fn emit_liquidity_withdrawn(
    env: &Env,
    provider: &Address,
    shares_burned: i128,
    amount_returned: i128,
) {
    env.events()
        .publish((WITHDRAWN, provider), (shares_burned, amount_returned));
}

/// Emitted when the pool funds a loan (CreditLine → merchant)
pub fn emit_loan_funded(env: &Env, creditline: &Address, amount: i128) {
    env.events().publish((LOAN_FUNDED, creditline), amount);
}

/// Emitted when principal + interest repayment is received from CreditLine
pub fn emit_repayment_received(env: &Env, creditline: &Address, principal: i128, interest: i128) {
    env.events()
        .publish((REPAYMENT_RCV, creditline), (principal, interest));
}

/// Emitted when a forfeited guarantee is received on loan default
pub fn emit_guarantee_received(env: &Env, creditline: &Address, amount: i128) {
    env.events()
        .publish((GUARANTEE_RCV, creditline), amount);
}

/// Emitted when interest is distributed to LPs, treasury, and merchant fund
pub fn emit_interest_distributed(
    env: &Env,
    total_interest: i128,
    lp_amount: i128,
    protocol_amount: i128,
    merchant_amount: i128,
) {
    env.events().publish(
        (INTEREST_DIST,),
        (total_interest, lp_amount, protocol_amount, merchant_amount),
    );
}
