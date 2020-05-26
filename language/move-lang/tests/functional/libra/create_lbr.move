//! account: bob, 10000000LBR

//! new-transaction
//! sender: bob
script {
use 0x0::LibraAccount;
use 0x0::Coin1;
use 0x0::Coin2;
// Setup bob's account as a multi-currency account.
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1::T>(account);
    LibraAccount::add_currency<Coin2::T>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::Coin1;
use 0x0::Coin2;
fun main() {
    LibraAccount::mint_to_address<Coin1::T>({{bob}}, 10000000);
    LibraAccount::mint_to_address<Coin2::T>({{bob}}, 10000000);
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
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
fun main() {
    let sender = Transaction::sender();
    let amount_lbr = 10;
    let coin1_balance = LibraAccount::balance<Coin1::T>({{bob}});
    let coin2_balance = LibraAccount::balance<Coin2::T>({{bob}});
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(coin1_balance);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(coin2_balance);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 10, 0);
    Transaction::assert(Libra::value(&coin1) == coin1_balance - 6, 1);
    Transaction::assert(Libra::value(&coin2) == coin2_balance - 6, 2);
    LibraAccount::deposit(sender, lbr);
    LibraAccount::deposit(sender, coin1);
    LibraAccount::deposit(sender, coin2);
}
}
// check: EXECUTED

// Now unpack from the LBR into the constituent coins
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(10);
    Transaction::assert(Libra::value(&lbr) == 10, 3);
    let (coin1, coin2) = LBR::unpack(lbr);
    Transaction::assert(Libra::value(&coin1) == 5, 4);
    Transaction::assert(Libra::value(&coin2) == 5, 5);
    LibraAccount::deposit({{bob}}, coin1);
    LibraAccount::deposit({{bob}}, coin2);
}
}
// not: PreburnEvent
// not: BurnEvent
// check: EXECUTED

// Now mint zero LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
fun main() {
    let amount_lbr = 0;
    let coin1_balance = LibraAccount::balance<Coin1::T>({{bob}});
    let coin2_balance = LibraAccount::balance<Coin2::T>({{bob}});
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(coin1_balance);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(coin2_balance);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 0, 6);
    Transaction::assert(Libra::value(&coin1) == coin1_balance, 7);
    Transaction::assert(Libra::value(&coin2) == coin2_balance, 8);
    Libra::destroy_zero(lbr);
    LibraAccount::deposit({{bob}}, coin1);
    LibraAccount::deposit({{bob}}, coin2);
}
}
// not: MintEvent
// check: EXECUTED
