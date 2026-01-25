use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

use crate::types::RepaymentInstallment;

// Event topics
const LOAN_CREATED: Symbol = symbol_short!("LOANCRTD");

/// Emit a loan created event
pub fn emit_loan_created(
    env: &Env,
    user: &Address,
    merchant: &Address,
    loan_id: u64,
    total_amount: i128,
    guarantee_amount: i128,
    repayment_schedule: &Vec<RepaymentInstallment>,
) {
    env.events().publish(
        (LOAN_CREATED, user, merchant),
        (
            loan_id,
            total_amount,
            guarantee_amount,
            repayment_schedule.clone(),
        ),
    );
}
