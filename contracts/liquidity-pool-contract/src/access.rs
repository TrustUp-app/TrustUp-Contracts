use soroban_sdk::{panic_with_error, Address, Env};

use crate::{storage, LiquidityPoolError};

pub fn require_admin(env: &Env, caller: &Address) {
    let admin = storage::get_admin(env);
    if admin != *caller {
        panic_with_error!(env, LiquidityPoolError::NotAdmin);
    }
}

pub fn require_creditline(env: &Env, caller: &Address) {
    let creditline = storage::get_creditline(env)
        .unwrap_or_else(|| panic_with_error!(env, LiquidityPoolError::NotCreditLine));
    if creditline != *caller {
        panic_with_error!(env, LiquidityPoolError::NotCreditLine);
    }
}
