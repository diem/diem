//! account: vivian, 1000000, 0, validator
//! account: dd, 0, 0, address
//! account: bob, 0XUS, 0, vasp

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
use 0x1::COIN;
fun main(dr_account: signer, tc_account: signer) {
    let dr_account = &dr_account;
    let tc_account = &tc_account;
    COIN::initialize(dr_account, tc_account);
}
}
// check: "Keep(EXECUTED)"


//! new-transaction
script {
use DiemFramework::TransactionFee;
use DiemFramework::Diem;
use 0x1::COIN::COIN;
fun main() {
    TransactionFee::pay_fee(Diem::zero<COIN>());
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::TransactionFee;
use 0x1::COIN::COIN;
fun main(tc: signer) {
    let tc = &tc;
    TransactionFee::burn_fees<COIN>(tc);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::TransactionFee;
use 0x1::COIN::COIN;
fun main(tc_account: signer) {
    let tc_account = &tc_account;
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(EXECUTED)"

// END: registration of a currency

// try adding a currency twice
//! new-transaction
//! sender: blessed
script {
use DiemFramework::TransactionFee;
use 0x1::COIN::COIN;
fun main(tc_account: signer) {
    let tc_account = &tc_account;
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::TransactionFee;
use DiemFramework::XDX::XDX;
fun main(tc: signer) {
    let tc = &tc;
    TransactionFee::add_txn_fee_currency<XDX>(tc);
    TransactionFee::burn_fees<XDX>(tc);
}
}
// check: "Keep(ABORTED { code: 1,"
