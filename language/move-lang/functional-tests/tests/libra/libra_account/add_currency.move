//! account: validator, 1000000, 0, validator
//! account: vasp, 0,0, address
//! account: child, 0,0, address
//! account: vivian, 0, 0, address
//! account: otto, 0, 0, address


// BEGIN: registration of a currency

//! new-transaction
//! sender: libraroot
// Change option to CustomModule
script {
use 0x1::LibraTransactionPublishingOption;
fun main(config: &signer) {
    LibraTransactionPublishingOption::set_open_module(config, false)
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: validator
//! block-time: 3


//! new-transaction
//! sender: libraroot
address 0x1 {
module COIN {
    use 0x1::FixedPoint32;
    use 0x1::Libra;

    struct COIN { }

    public fun initialize(lr_account: &signer, tc_account: &signer) {
        // Register the COIN currency.
        Libra::register_SCS_currency<COIN>(
            lr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"COIN",
        )
    }
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: validator
//! block-time: 4

//! new-transaction
//! sender: libraroot
//! execute-as: blessed
script {
use 0x1::TransactionFee;
use 0x1::COIN::{Self, COIN};
fun main(lr_account: &signer, tc_account: &signer) {
    COIN::initialize(lr_account, tc_account);
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(EXECUTED)"

// END: registration of a currency

// LibraRoot should not be able to add a balance
//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(ABORTED { code: 1031,"

// TreasuryCompliance should not be able to add a balance
//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(ABORTED { code: 1031,"


// Validators and ValidatorOperators should not be able to add a balance
//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::create_validator_account(account, {{vivian}}, {{vivian::auth_key}}, b"owner_name");
    LibraAccount::create_validator_operator_account(account, {{otto}}, {{otto::auth_key}}, b"operator_name")

}
}
// check: "Keep(EXECUTED)"

// check Validator case
//! new-transaction
//! sender: vivian
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(ABORTED { code: 1031,"

// check ValidatorOperator case
//! new-transaction
//! sender: otto
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(ABORTED { code: 1031,"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{vasp}}, {{vasp::auth_key}}, b"bob", false
stdlib_script::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vasp
//! type-args: 0x1::COIN::COIN
stdlib_script::add_currency_to_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vasp
//! type-args: 0x1::COIN::COIN
//! args: {{child}}, {{child::auth_key}}, false, 0
stdlib_script::create_child_vasp_account
// check: "Keep(EXECUTED)"

// can't add a balance of LBR right now
//! new-transaction
//! sender: child
//! type-args: 0x1::LBR::LBR
stdlib_script::add_currency_to_account
// check: "Keep(EXECUTED)"
