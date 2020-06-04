//! account: bob, 100Coin1

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
use 0x0::LibraAccount;
use 0x0::LBR::LBR;
use 0x0::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
    LibraAccount::add_currency<LBR>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::mint_to_address<Coin2>(account, {{bob}}, 100);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
//! gas-currency: Coin1
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main(sender: &signer) {
    let coin1 = LibraAccount::withdraw_from<Coin1>(sender, 10);
    let coin2 = LibraAccount::withdraw_from<Coin2>(sender, 10);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 18, 0);
    LibraAccount::deposit_to(sender, lbr);
    Libra::destroy_zero(coin1);
    Libra::destroy_zero(coin2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
//! gas-currency: Coin1
script {
use 0x0::LBR::{Self, LBR};
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
fun main(sender: &signer) {
    let lbr = LibraAccount::withdraw_from<LBR>(sender, 18);
    let (coin1, coin2) = LBR::unpack(sender, lbr);
    Transaction::assert(Libra::value(&coin1) == 9, 1);
    Transaction::assert(Libra::value(&coin2) == 9, 2);
    LibraAccount::deposit_to(sender, coin1);
    LibraAccount::deposit_to(sender, coin2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main(sender: &signer) {
    let coin1 = LibraAccount::withdraw_from<Coin1>(sender, 2);
    let coin2 = LibraAccount::withdraw_from<Coin2>(sender, 1);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 0, 0);
    Transaction::assert(Libra::value(&coin1) == 2, 1);
    Transaction::assert(Libra::value(&coin2) == 1, 2);
    LibraAccount::deposit_to(sender, coin1);
    LibraAccount::deposit_to(sender, coin2);
    Libra::destroy_zero(lbr);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main(sender: &signer) {
    let coin1 = LibraAccount::withdraw_from<Coin1>(sender, 1);
    let coin2 = LibraAccount::withdraw_from<Coin2>(sender, 2);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 0, 0);
    Transaction::assert(Libra::value(&coin1) == 1, 1);
    Transaction::assert(Libra::value(&coin2) == 2, 2);
    LibraAccount::deposit_to(sender, coin1);
    LibraAccount::deposit_to(sender, coin2);
    Libra::destroy_zero(lbr);
}
}
// check: EXECUTED

// Now mint LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main(sender: &signer) {
    let coin1 = LibraAccount::withdraw_from<Coin1>(sender, 9);
    let coin2 = LibraAccount::withdraw_from<Coin2>(sender, 10);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 16, 0);
    LibraAccount::deposit_to(sender, lbr);
    Libra::destroy_zero(coin1);
    LibraAccount::deposit_to(sender, coin2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use 0x0::LBR;
fun main(sender: &signer) {
    let coin1 = LibraAccount::withdraw_from<Coin1>(sender, 10);
    let coin2 = LibraAccount::withdraw_from<Coin2>(sender, 9);
    let (lbr, coin1, coin2) = LBR::swap_into(coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 16, 0);
    LibraAccount::deposit_to(sender, lbr);
    LibraAccount::deposit_to(sender, coin1);
    Libra::destroy_zero(coin2);
}
}
// check: EXECUTED
