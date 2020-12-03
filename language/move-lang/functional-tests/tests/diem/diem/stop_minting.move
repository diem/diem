//! account: vivian, 1000000, 0, validator
//! account: dd1, 0, 0, address
//! account: dd2, 0, 0, address

// BEGIN: registration of a currency

//! new-transaction
//! sender: diemroot
// Change option to CustomModule
script {
use 0x1::DiemTransactionPublishingOption;
fun main(config: &signer) {
    DiemTransactionPublishingOption::set_open_module(config, false)
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: vivian
//! block-time: 3


//! new-transaction
//! sender: diemroot
address 0x1 {
module COIN {
    use 0x1::FixedPoint32;
    use 0x1::Diem;

    struct COIN { }

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
use 0x1::TransactionFee;
use 0x1::COIN::{Self, COIN};
fun main(dr_account: &signer, tc_account: &signer) {
    COIN::initialize(dr_account, tc_account);
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(EXECUTED)"

// END: registration of a currency

//! new-transaction
//! sender: blessed
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
use 0x1::COIN::COIN;
use 0x1::Diem;

// register dd(1|2) as a preburner
fun main(account: &signer) {
    let prev_mcap1 = Diem::market_cap<XUS>();
    let prev_mcap2 = Diem::market_cap<COIN>();
    DiemAccount::create_designated_dealer<XUS>(
        account,
        {{dd1}},
        {{dd1::auth_key}},
        x"",
        false,
    );
    DiemAccount::create_designated_dealer<COIN>(
        account,
        {{dd2}},
        {{dd2::auth_key}},
        x"",
        false,
    );
    DiemAccount::tiered_mint<XUS>(
        account,
        {{dd1}},
        10,
        0,
    );
    DiemAccount::tiered_mint<COIN>(
        account,
        {{dd2}},
        100,
        0,
    );
    assert(Diem::market_cap<XUS>() - prev_mcap1 == 10, 7);
    assert(Diem::market_cap<COIN>() - prev_mcap2 == 100, 8);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dd1
script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::preburn<XUS>(account, &with_cap, 10);
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dd2
script {
use 0x1::COIN::COIN;
use 0x1::DiemAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::preburn<COIN>(account, &with_cap, 100);
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"


// do some burning
//! new-transaction
//! sender: blessed
script {
use 0x1::Diem;
use 0x1::XUS::XUS;
use 0x1::COIN::COIN;

fun main(account: &signer) {
    let prev_mcap1 = Diem::market_cap<XUS>();
    let prev_mcap2 = Diem::market_cap<COIN>();
    Diem::burn<XUS>(account, {{dd1}});
    Diem::burn<COIN>(account, {{dd2}});
    assert(prev_mcap1 - Diem::market_cap<XUS>() == 10, 9);
    assert(prev_mcap2 - Diem::market_cap<COIN>() == 100, 10);
}
}
// check: "Keep(EXECUTED)"

// check that stop minting works
//! new-transaction
//! sender: blessed
script {
use 0x1::Diem;
use 0x1::XUS::XUS;

fun main(account: &signer) {
    Diem::update_minting_ability<XUS>(account, false);
    let coin = Diem::mint<XUS>(account, 10); // will abort here
    Diem::destroy_zero(coin);
}
}
// check: "Keep(ABORTED { code: 1281,"
