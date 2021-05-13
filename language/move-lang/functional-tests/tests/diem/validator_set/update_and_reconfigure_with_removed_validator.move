// Make bob a validator, set alice as bob's delegate.
// Test that alice can rotate bob's key and invoke reconfiguration.

//! account: alice, 0, 0, address
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! new-transaction
//! sender: diemroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, @{{alice}});
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
// remove_validator cannot be called on a non-validator
script{
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{bob}});
    }
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: carrol
//! block-time: 2

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
//! expiration-time: 3
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}
// check: "ABORTED { code: 775,"
