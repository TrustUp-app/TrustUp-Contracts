#![cfg(test)]

use crate::setup::TestEnv;
use soroban_sdk::{testutils::Address as _, Address};

#[test]
fn test_unauthorized_access() {
    let setup = TestEnv::setup();
    let env = &setup.env;

    let malicious = Address::generate(env);
    let target = Address::generate(env);
    
    // Malicious user trying to increase score
    // Since we mock all auths by default in setup(), `mock_all_auths()` means require_auth() will pass!
    // But `require_updater()` explicitly checks if the address is registered as an updater.
    let res = setup.reputation.try_increase_score(&malicious, &target, &10);
    assert!(res.is_err());
}
