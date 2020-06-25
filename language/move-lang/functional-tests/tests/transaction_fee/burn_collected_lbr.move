//! account: bob, 0LBR

//! new-transaction
//! sender: blessed
script {
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR;
    use 0x1::Libra;
    use 0x1::LibraAccount;
    fun main(assoc: &signer) {
        let amount = 100000;
        let coin1 = Libra::mint<Coin1>(assoc, amount);
        let coin2 = Libra::mint<Coin2>(assoc, amount);
        let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
        Libra::destroy_zero(coin1);
        Libra::destroy_zero(coin2);
        LibraAccount::deposit(assoc, {{bob}}, lbr);
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
use 0x1::TransactionFee;
use 0x1::Roles::{Self, TreasuryComplianceRole};
fun burn_txn_fees<CoinType>(blessed_account: &signer) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(blessed_account);
    TransactionFee::burn_fees<CoinType>(blessed_account, &tc_capability);
    Roles::restore_capability_to_privilege(blessed_account, tc_capability);
}
}
// check: PreburnEvent
// check: BurnEvent
// check: PreburnEvent
// check: BurnEvent
// check: EXECUTED
