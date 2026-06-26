use crate::types::MerchantMetadata;
use soroban_sdk::{Address, Env, Symbol};

pub fn publish_merchant_registered(env: &Env, merchant: Address, metadata: MerchantMetadata) {
    let topics = (Symbol::new(env, "MERCHTREG"), merchant);
    env.events().publish(topics, metadata);
}

pub fn publish_merchant_status(env: &Env, merchant: Address, active: bool) {
    let topics = (Symbol::new(env, "MERCHTSTATUS"), merchant);
    env.events().publish(topics, active);
}

