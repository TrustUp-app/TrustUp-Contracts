use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

use crate::types::RepaymentInstallment;

// Event topics
const LOAN_CREATED: Symbol = symbol_short!("LOANCRTD");
const LOAN_DEFAULTED: Symbol = symbol_short!("LOANDFLT");
const LOAN_REPAID: Symbol = symbol_short!("LOANRPD");

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

pub fn emit_loan_defaulted(
    env: &Env,
    borrower: Address,
    loan_id: u64,
    total_amount: i128,
    unpaid_balance: i128,
    guarantee_forfeited: i128,
) {
    env.events().publish(
        (LOAN_DEFAULTED, borrower, loan_id),
        (
            total_amount,
            unpaid_balance,
            guarantee_forfeited,
            env.ledger().timestamp(),
        ),
    );
}

pub fn emit_loan_repaid(
    env: &Env,
    borrower: &Address,
    loan_id: u64,
    amount_paid: i128,
    remaining_balance: i128,
    is_fully_repaid: bool,
) {
    env.events().publish(
        (LOAN_REPAID, borrower, loan_id),
        (
            amount_paid,
            remaining_balance,
            is_fully_repaid,
            env.ledger().timestamp(),
        ),
    );
}
