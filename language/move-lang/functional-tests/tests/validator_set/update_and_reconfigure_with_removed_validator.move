// Make bob a validator, set alice as bob's delegate.
// Test that alice can rotate bob's key and invoke reconfiguration.

//! account: alice, 0, 0, address
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! new-transaction
//! sender: libraroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::create_validator_operator_account
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, {{alice}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
// remove_validator cannot be called on a non-validator
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{bob}});
    }
}
// check: EXECUTED

//! block-prologue
//! proposer: carrol
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: alice
//! expiration-time: 3
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::update_config_and_reconfigure(account, {{bob}});
    }
}
// check: "ABORTED { code: 5,"
