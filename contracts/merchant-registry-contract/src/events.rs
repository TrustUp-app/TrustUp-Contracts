use soroban_sdk::{Address, Env, String, Symbol};

pub fn publish_merchant_registered(env: &Env, merchant: Address, name: String) {
    let topics = (Symbol::new(env, "MERCHTREG"), merchant);
    env.events().publish(topics, name);
}

pub fn publish_merchant_status(env: &Env, merchant: Address, active: bool) {
    let topics = (Symbol::new(env, "MERCHTSTATUS"), merchant);
    env.events().publish(topics, active);
}
