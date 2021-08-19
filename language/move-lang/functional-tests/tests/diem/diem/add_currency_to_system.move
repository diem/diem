//! account: vivian, 1000000, 0, validator
//! account: dd, 0, 0, address
//! account: bob, 0XUS, 0, vasp

//! new-transaction
//! sender: bob
//! gas-currency: COIN
script {
fun main() {}
}
// check: "Discard(CURRENCY_INFO_DOES_NOT_EXIST)"

//! block-prologue
//! proposer: vivian
//! block-time: 2

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
//! proposer: vivian
//! block-time: 3

// BEGIN: registration of a currency

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
//! proposer: vivian
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
//! gas-currency: COIN
script {
use DiemFramework::Diem;
use 0x1::COIN::COIN;
use Std::FixedPoint32;
fun main(account: signer) {
    let account = &account;
    assert(Diem::approx_xdx_for_value<COIN>(10) == 5, 1);
    assert(Diem::scaling_factor<COIN>() == 1000000, 2);
    assert(Diem::fractional_part<COIN>() == 100, 3);
    Diem::update_xdx_exchange_rate<COIN>(account, FixedPoint32::create_from_rational(1, 3));
    assert(Diem::approx_xdx_for_value<COIN>(10) == 3, 4);
}
}
// check: ToXDXExchangeRateUpdateEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::DiemAccount;
use 0x1::COIN::COIN;
use DiemFramework::Diem;
fun main(account: signer) {
    let account = &account;
    let prev_mcap3 = Diem::market_cap<COIN>();
    DiemAccount::create_designated_dealer<COIN>(
        account,
        @{{dd}},
        {{dd::auth_key}},
        x"",
        false,
    );
    DiemAccount::tiered_mint<COIN>(
        account,
        @{{dd}},
        10000,
        0,
    );
    assert(Diem::market_cap<COIN>() - prev_mcap3 == 10000, 8);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
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
//! sender: dd
script {
use DiemFramework::DiemAccount;
use 0x1::COIN::COIN;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<COIN>(
        &with_cap,
        @{{bob}},
        10000,
        x"",
        x""
    );
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! gas-currency: COIN
//! gas-price: 1
script {
fun main() {}
}
// check: "Keep(EXECUTED)"
