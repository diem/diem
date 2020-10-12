//! account: vivian, 1000000, 0, validator
//! account: dd, 0, 0, address
//! account: bob, 0Coin1, 0, vasp

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
use 0x1::COIN;
fun main(lr_account: &signer, tc_account: &signer) {
    COIN::initialize(lr_account, tc_account);
}
}
// check: "Keep(EXECUTED)"


//! new-transaction
script {
use 0x1::TransactionFee;
use 0x1::Libra;
use 0x1::COIN::COIN;
fun main() {
    TransactionFee::pay_fee(Libra::zero<COIN>());
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: blessed
script {
use 0x1::TransactionFee;
use 0x1::COIN::COIN;
fun main(tc: &signer) {
    TransactionFee::burn_fees<COIN>(tc);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: blessed
script {
use 0x1::TransactionFee;
use 0x1::COIN::COIN;
fun main(tc_account: &signer) {
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(EXECUTED)"

// END: registration of a currency

// try adding a currency twice
//! new-transaction
//! sender: blessed
script {
use 0x1::TransactionFee;
use 0x1::COIN::COIN;
fun main(tc_account: &signer) {
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: blessed
script {
use 0x1::TransactionFee;
use 0x1::LBR::LBR;
fun main(tc: &signer) {
    TransactionFee::add_txn_fee_currency<LBR>(tc);
    TransactionFee::burn_fees<LBR>(tc);
}
}
// check: "Keep(ABORTED { code: 1,"
