//! account: bob, 10000Coin1

//! new-transaction
//! sender: bob
//! max-gas: 1000
//! gas-price: 1
//! gas-currency: Coin1
script {
    fun main() { while (true) {} }
}
// check: OUT_OF_GAS

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
script {
use 0x1::TransactionFee;
fun burn_txn_fees<CoinType>(blessed_account: &signer) {
    TransactionFee::burn_fees<CoinType>(blessed_account);
}
}
// check: PreburnEvent
// check: BurnEvent
// check: "Keep(EXECUTED)"

// No txn fee balance left to burn

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
script {
use 0x1::TransactionFee;
fun burn_txn_fees<CoinType>(blessed_account: &signer) {
    TransactionFee::burn_fees<CoinType>(blessed_account);
}
}

// check: "Keep(ABORTED { code: 1799,"
