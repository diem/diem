//! account: vivian, 1000000, 0, validator
//! account: dd1, 0, 0, address
//! account: dd2, 0, 0, address

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
//! proposer: vivian
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
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::COIN::COIN;
use 0x1::Libra;

// register dd(1|2) as a preburner
fun main(account: &signer) {
    let prev_mcap1 = Libra::market_cap<Coin1>();
    let prev_mcap2 = Libra::market_cap<COIN>();
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        {{dd1}},
        {{dd1::auth_key}},
        x"",
        false,
    );
    LibraAccount::create_designated_dealer<COIN>(
        account,
        {{dd2}},
        {{dd2::auth_key}},
        x"",
        false,
    );
    LibraAccount::tiered_mint<Coin1>(
        account,
        {{dd1}},
        10,
        0,
    );
    LibraAccount::tiered_mint<COIN>(
        account,
        {{dd2}},
        100,
        0,
    );
    assert(Libra::market_cap<Coin1>() - prev_mcap1 == 10, 7);
    assert(Libra::market_cap<COIN>() - prev_mcap2 == 100, 8);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dd1
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::preburn<Coin1>(account, &with_cap, 10);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dd2
script {
use 0x1::COIN::COIN;
use 0x1::LibraAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::preburn<COIN>(account, &with_cap, 100);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"


// do some burning
//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::COIN::COIN;

fun main(account: &signer) {
    let prev_mcap1 = Libra::market_cap<Coin1>();
    let prev_mcap2 = Libra::market_cap<COIN>();
    Libra::burn<Coin1>(account, {{dd1}});
    Libra::burn<COIN>(account, {{dd2}});
    assert(prev_mcap1 - Libra::market_cap<Coin1>() == 10, 9);
    assert(prev_mcap2 - Libra::market_cap<COIN>() == 100, 10);
}
}
// check: "Keep(EXECUTED)"

// check that stop minting works
//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;

fun main(account: &signer) {
    Libra::update_minting_ability<Coin1>(account, false);
    let coin = Libra::mint<Coin1>(account, 10); // will abort here
    Libra::destroy_zero(coin);
}
}
// check: "Keep(ABORTED { code: 1281,"
