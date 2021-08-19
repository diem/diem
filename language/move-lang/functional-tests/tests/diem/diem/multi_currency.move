//! account: alice, 0, 0, address
//! account: bob, 0, 0, address
//! account: richie, 10XUS
//! account: sally, 0, 0, address

// BEGIN: registration of a currency
//! account: validator, 1000000, 0, validator

//! new-transaction
//! sender: diemroot
// Change option to CustomModule
script {
use DiemFramework::DiemTransactionPublishingOption;
fun main(config: signer) {
    let config = &config;
    DiemTransactionPublishingOption::set_open_module(config, false)
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: validator
//! block-time: 3


//! new-transaction
//! sender: diemroot
address 0x1 {
module COIN {
    use Std::FixedPoint32;
    use DiemFramework::Diem;

    struct COIN has store { }

    public fun initialize(dr_account: &signer, tc_account: &signer) {
        // Register the COIN currency.
        Diem::register_SCS_currency<COIN>(
            dr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to XDX
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
//! sender: diemroot
//! execute-as: blessed
script {
use DiemFramework::TransactionFee;
use 0x1::COIN::{Self, COIN};
fun main(dr_account: signer, tc_account: signer) {
    let dr_account = &dr_account;
    let tc_account = &tc_account;
    COIN::initialize(dr_account, tc_account);
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(EXECUTED)"

// END: registration of a currency

//! new-transaction
//! sender: blessed
//! type-args: 0x1::COIN::COIN
//! args: 0, {{sally}}, {{sally::auth_key}}, b"bob", false
stdlib_script::AccountCreationScripts::create_designated_dealer
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::COIN::COIN;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    DiemAccount::tiered_mint<COIN>(account, @{{sally}}, 10, 3);
}
}

// create parent VASP accounts for alice and bob
//! new-transaction
//! sender: blessed
script {
use DiemFramework::XUS::XUS;
use 0x1::COIN::COIN;
use DiemFramework::DiemAccount;
fun main(tc_account: signer) {
    let tc_account = &tc_account;
    let add_all_currencies = false;

    DiemAccount::create_parent_vasp_account<XUS>(
        tc_account,
        @{{alice}},
        {{alice::auth_key}},
        x"A1",
        add_all_currencies,
    );

    DiemAccount::create_parent_vasp_account<COIN>(
        tc_account,
        @{{bob}},
        {{bob::auth_key}},
        x"B1",
        add_all_currencies,
    );
}
}
// check: "Keep(EXECUTED)"

// Give alice money from richie
//! new-transaction
//! sender: richie
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 10, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

// Give bob money from sally
//! new-transaction
//! sender: sally
script {
use DiemFramework::DiemAccount;
use 0x1::COIN::COIN;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<COIN>(&with_cap, @{{bob}}, 10, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use DiemFramework::DiemAccount;
use 0x1::COIN::COIN;
fun main(account: signer) {
    let account = &account;
    DiemAccount::add_currency_for_test<COIN>(account);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    DiemAccount::add_currency_for_test<XUS>(account);
}
}
// check: "Keep(EXECUTED)"

// Adding a bogus currency should abort
//! new-transaction
//! sender: alice
script {
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    DiemAccount::add_currency_for_test<u64>(account);
}
}
// check: "Keep(ABORTED { code: 261,"

// Adding XUS a second time should fail with ADD_EXISTING_CURRENCY
//! new-transaction
//! sender: alice
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    DiemAccount::add_currency_for_test<XUS>(account);
}
}
// check: "Keep(ABORTED { code: 3846,"

//! new-transaction
//! sender: alice
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 10, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
    assert(DiemAccount::balance<XUS>(@{{alice}}) == 0, 0);
    assert(DiemAccount::balance<XUS>(@{{bob}}) == 10, 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use DiemFramework::DiemAccount;
use 0x1::COIN::COIN;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<COIN>(&with_cap, @{{alice}}, 10, x"", x"");
    DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 10, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
    assert(DiemAccount::balance<XUS>(@{{bob}}) == 0, 2);
    assert(DiemAccount::balance<COIN>(@{{bob}}) == 0, 3);
    assert(DiemAccount::balance<XUS>(@{{alice}}) == 10, 4);
    assert(DiemAccount::balance<COIN>(@{{alice}}) == 10, 5);
}
}
// check: "Keep(EXECUTED)"
