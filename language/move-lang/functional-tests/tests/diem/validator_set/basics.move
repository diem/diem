//! account: bob, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: alice, 0, 0, address
//! account: alex, 0, 0, address

//! new-transaction
script {
use DiemFramework::DiemSystem;
fun main(account: signer) {
    let account = &account;
    DiemSystem::initialize_validator_set(account);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
script {
use DiemFramework::DiemSystem;
fun main() {
    let len = DiemSystem::validator_set_size();
    DiemSystem::get_ith_validator_address(len);
}
}
// check: "Keep(ABORTED { code: 1287,"

//! new-transaction
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
        let account = &account;
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}
// check: "Keep(ABORTED { code: 2051,"

//! new-transaction
//! sender: diemroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: CreateAccountEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
//! args: 0, {{alex}}, {{alex::auth_key}}, b"alex"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: "Keep(EXECUTED)"
