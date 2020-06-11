//! account: bob, 0LBR

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    fun main(assoc: &signer) {
        LibraAccount::mint_lbr_to_address(assoc, {{bob}}, 100000);
    }
}
// check: EXECUTED

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
use 0x1::Libra;
use 0x1::TransactionFee;
use 0x1::Coin1::Coin1 as Coin1;
use 0x1::Coin2::Coin2 as Coin2;
fun main<CoinType>(blessed_account: &signer) {
    TransactionFee::preburn_fees<CoinType>(blessed_account);
    if (TransactionFee::is_lbr<CoinType>()) {
        let coin1_burn_cap = Libra::remove_burn_capability<Coin1>(blessed_account);
        let coin2_burn_cap = Libra::remove_burn_capability<Coin2>(blessed_account);
        TransactionFee::burn_fees(&coin1_burn_cap);
        TransactionFee::burn_fees(&coin2_burn_cap);
        Libra::publish_burn_capability(blessed_account, coin1_burn_cap);
        Libra::publish_burn_capability(blessed_account, coin2_burn_cap);
    } else {
        let burn_cap = Libra::remove_burn_capability<CoinType>(blessed_account);
        TransactionFee::burn_fees(&burn_cap);
        Libra::publish_burn_capability(blessed_account, burn_cap);
    }
}
}
// check: BurnEvent
// check: BurnEvent
// not: BurnEvent
// check: EXECUTED
