//! account: bob, 10Coin1

//! new-transaction
//! sender: bob
script {
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Coin2;
fun main() {
    LibraAccount::add_currency<Coin2::T>();
    LibraAccount::add_currency<LBR::T>();
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
use 0x0::Coin2;
fun main() {
    LibraAccount::mint_to_address<Coin2::T>({{bob}}, 10);
}
}
// check: EXECUTED

// Now mint LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main() {
    let sender = Transaction::sender();
    let coin1_balance = LibraAccount::balance<Coin1::T>({{bob}});
    let coin2_balance = LibraAccount::balance<Coin2::T>({{bob}});
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(coin1_balance);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(coin2_balance);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 18, 0);
    LibraAccount::deposit(sender, lbr);
    Libra::destroy_zero(coin1);
    Libra::destroy_zero(coin2);
}
}
// check: EXECUTED

// Now mint LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(18);
    let (coin1, coin2) = LBR::unpack(lbr);
    Transaction::assert(Libra::value(&coin1) == 9, 1);
    Transaction::assert(Libra::value(&coin2) == 9, 2);
    LibraAccount::deposit({{bob}}, coin1);
    LibraAccount::deposit({{bob}}, coin2);
}
}
// check: EXECUTED
