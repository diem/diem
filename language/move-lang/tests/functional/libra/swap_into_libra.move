//! account: bob, 100Coin1

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2::T>(account);
    LibraAccount::add_currency<LBR::T>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::Coin2;
fun main() {
    LibraAccount::mint_to_address<Coin2::T>({{bob}}, 100);
}
}
// check: EXECUTED

// Now mint LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
//! gas-currency: Coin1
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main() {
    let sender = Transaction::sender();
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(10);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(10);
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
//! gas-currency: Coin1
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
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(2);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(1);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 0, 0);
    Transaction::assert(Libra::value(&coin1) == 2, 1);
    Transaction::assert(Libra::value(&coin2) == 1, 2);
    LibraAccount::deposit(sender, coin1);
    LibraAccount::deposit(sender, coin2);
    Libra::destroy_zero(lbr);
}
}
// check: EXECUTED

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
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(1);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(2);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 0, 0);
    Transaction::assert(Libra::value(&coin1) == 1, 1);
    Transaction::assert(Libra::value(&coin2) == 2, 2);
    LibraAccount::deposit(sender, coin1);
    LibraAccount::deposit(sender, coin2);
    Libra::destroy_zero(lbr);
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
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(9);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(10);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 16, 0);
    LibraAccount::deposit(sender, lbr);
    Libra::destroy_zero(coin1);
    LibraAccount::deposit(sender, coin2);
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
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(10);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(9);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 16, 0);
    LibraAccount::deposit(sender, lbr);
    LibraAccount::deposit(sender, coin1);
    Libra::destroy_zero(coin2);
}
}
// check: EXECUTED
