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
// check: ABORTED
// check: 4

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
// check: ABORTED
// check: 4


// Validators and ValidatorOperators should not be able to add a balance
//! account: vivian, 0, 0, address
//! account: otto, 0, 0, address

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::create_validator_account(account, {{vivian}}, {{vivian::auth_key}});
    LibraAccount::create_validator_operator_account(account, {{otto}}, {{otto::auth_key}})

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
// check: ABORTED
// check: 4

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
// check: ABORTED
// check: 4
