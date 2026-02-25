use crate::types::{DataKey, MerchantInfo};
use soroban_sdk::{Address, Env};

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn has_merchant(env: &Env, merchant: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Merchant(merchant.clone()))
}

pub fn get_merchant(env: &Env, merchant: &Address) -> Option<MerchantInfo> {
    env.storage()
        .persistent()
        .get(&DataKey::Merchant(merchant.clone()))
}

pub fn set_merchant(env: &Env, merchant: &Address, info: &MerchantInfo) {
    env.storage()
        .persistent()
        .set(&DataKey::Merchant(merchant.clone()), info);
}

pub fn get_merchant_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantCount)
        .unwrap_or(0)
}

pub fn increment_merchant_count(env: &Env) {
    let count = get_merchant_count(env);
    env.storage()
        .persistent()
        .set(&DataKey::MerchantCount, &(count + 1));
}
