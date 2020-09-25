//! account: vivian, 1000000, 0, validator
//! account: dd, 0, 0, address
//! account: bob, 0Coin1, 0, vasp

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
//! proposer: vivian
//! block-time: 3

// BEGIN: registration of a currency

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
//! proposer: vivian
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

//! new-transaction
//! sender: blessed
//! gas-currency: COIN
script {
use 0x1::Libra;
use 0x1::COIN::COIN;
use 0x1::FixedPoint32;
fun main(account: &signer) {
    assert(Libra::approx_lbr_for_value<COIN>(10) == 5, 1);
    assert(Libra::scaling_factor<COIN>() == 1000000, 2);
    assert(Libra::fractional_part<COIN>() == 100, 3);
    Libra::update_lbr_exchange_rate<COIN>(account, FixedPoint32::create_from_rational(1, 3));
    assert(Libra::approx_lbr_for_value<COIN>(10) == 3, 4);
}
}
// check: ToLBRExchangeRateUpdateEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::COIN::COIN;
use 0x1::Libra;
fun main(account: &signer) {
    let prev_mcap3 = Libra::market_cap<COIN>();
    LibraAccount::create_designated_dealer<COIN>(
        account,
        {{dd}},
        {{dd::auth_key}},
        x"",
        false,
    );
    LibraAccount::tiered_mint<COIN>(
        account,
        {{dd}},
        10000,
        0,
    );
    assert(Libra::market_cap<COIN>() - prev_mcap3 == 10000, 8);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::COIN::COIN;
fun main(account: &signer) {
    LibraAccount::add_currency<COIN>(account);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dd
script {
use 0x1::LibraAccount;
use 0x1::COIN::COIN;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<COIN>(
        &with_cap,
        {{bob}},
        10000,
        x"",
        x""
    );
    LibraAccount::restore_withdraw_capability(with_cap);
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
