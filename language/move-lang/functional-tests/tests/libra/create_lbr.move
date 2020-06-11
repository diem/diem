//! account: bob, 10000000LBR

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
// Setup bob's account as a multi-currency account.
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
    LibraAccount::add_currency<Coin2>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::mint_to_address<Coin1>(account, {{bob}}, 10000000);
    LibraAccount::mint_to_address<Coin2>(account, {{bob}}, 10000000);
}
}
// check: EXECUTED

// Now mint LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LBR;
use 0x1::LibraAccount;
use 0x1::Libra;
fun main(account: &signer) {
    let amount_lbr = 10;
    let coin1_balance = LibraAccount::balance<Coin1>({{bob}});
    let coin2_balance = LibraAccount::balance<Coin2>({{bob}});
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin1 = LibraAccount::withdraw_from<Coin1>(&with_cap, coin1_balance);
    let coin2 = LibraAccount::withdraw_from<Coin2>(&with_cap, coin2_balance);
    LibraAccount::restore_withdraw_capability(with_cap);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    assert(Libra::value(&lbr) == 10, 0);
    assert(Libra::value(&coin1) == coin1_balance - 6, 1);
    assert(Libra::value(&coin2) == coin2_balance - 6, 2);
    LibraAccount::deposit_to(account, lbr);
    LibraAccount::deposit_to(account, coin1);
    LibraAccount::deposit_to(account, coin2);
}
}
// check: EXECUTED

// Now unpack from the LBR into the constituent coins
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x1::LBR::{Self, LBR};
use 0x1::LibraAccount;
use 0x1::Libra;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let lbr = LibraAccount::withdraw_from<LBR>(&with_cap, 10);
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(Libra::value(&lbr) == 10, 3);
    let (coin1, coin2) = LBR::unpack(account, lbr);
    assert(Libra::value(&coin1) == 5, 4);
    assert(Libra::value(&coin2) == 5, 5);
    LibraAccount::deposit(account, {{bob}}, coin1);
    LibraAccount::deposit(account, {{bob}}, coin2);
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
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LBR;
use 0x1::LibraAccount;
use 0x1::Libra;
fun main(account: &signer) {
    let amount_lbr = 0;
    let coin1_balance = LibraAccount::balance<Coin1>({{bob}});
    let coin2_balance = LibraAccount::balance<Coin2>({{bob}});
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin1 = LibraAccount::withdraw_from<Coin1>(&with_cap, coin1_balance);
    let coin2 = LibraAccount::withdraw_from<Coin2>(&with_cap, coin2_balance);
    LibraAccount::restore_withdraw_capability(with_cap);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    assert(Libra::value(&lbr) == 0, 6);
    assert(Libra::value(&coin1) == coin1_balance, 7);
    assert(Libra::value(&coin2) == coin2_balance, 8);
    Libra::destroy_zero(lbr);
    LibraAccount::deposit(account, {{bob}}, coin1);
    LibraAccount::deposit(account, {{bob}}, coin2);
}
}
// not: MintEvent
// check: EXECUTED
