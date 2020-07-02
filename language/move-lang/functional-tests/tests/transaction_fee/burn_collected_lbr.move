//! account: bob, 100000LBR

//! new-transaction
//! sender: bob
//! max-gas: 1000
//! gas-price: 1
script {
    fun main() { while (true) {} }
}
// check: OUT_OF_GAS

//! new-transaction
//! sender: blessed
//! type-args: 0x1::LBR::LBR
script {
use 0x1::TransactionFee;
fun burn_txn_fees<CoinType>(blessed_account: &signer) {
    TransactionFee::burn_fees<CoinType>(blessed_account);
}
}
// check: PreburnEvent
// check: BurnEvent
// check: PreburnEvent
// check: BurnEvent
// check: EXECUTED
