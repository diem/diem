//! account: vasp, 0,0, address
//! account: child, 0,0, address

// LibraRoot should not be able to add a balance
//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 1031

// TreasuryCompliance should not be able to add a balance
//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 1031


// Validators and ValidatorOperators should not be able to add a balance
//! account: vivian, 0, 0, address
//! account: otto, 0, 0, address

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::create_validator_account(account, {{vivian}}, {{vivian::auth_key}}, b"owner_name");
    LibraAccount::create_validator_operator_account(account, {{otto}}, {{otto::auth_key}}, b"operator_name")

}
}
// check: EXECUTED

// check Validator case
//! new-transaction
//! sender: vivian
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 1031

// check ValidatorOperator case
//! new-transaction
//! sender: otto
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 1031

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{vasp}}, {{vasp::auth_key}}, b"bob", false
stdlib_script::create_parent_vasp_account
// check: EXECUTED

//! new-transaction
//! sender: vasp
//! type-args: 0x1::Coin2::Coin2
stdlib_script::add_currency_to_account
// check: EXECUTED

//! new-transaction
//! sender: vasp
//! type-args: 0x1::Coin2::Coin2
//! args: {{child}}, {{child::auth_key}}, false, 0
stdlib_script::create_child_vasp_account
// check: EXECUTED

//! new-transaction
//! sender: child
//! type-args: 0x1::LBR::LBR
stdlib_script::add_currency_to_account
// check: EXECUTED
