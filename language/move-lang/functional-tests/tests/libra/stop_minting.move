//! new-transaction
//! sender: association
script {
use 0x0::Libra;
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
//use 0x0::Signer;
use 0x0::Transaction;

// do some preburning
fun main(account: &signer) {
    Libra::publish_preburn_to_account<Coin1>(account, account);
    Libra::publish_preburn_to_account<Coin2>(account, account);
    let coin1_coins = Libra::mint<Coin1>(account, 10);
    let coin2_coins = Libra::mint<Coin2>(account, 10);
    Transaction::assert(Libra::market_cap<Coin1>() == 10, 7);
    Transaction::assert(Libra::market_cap<Coin2>() == 10, 8);
    Libra::preburn_to(account, coin1_coins);
    Libra::preburn_to(account, coin2_coins);
}
}
// check: EXECUTED

// do some burning
//! new-transaction
//! sender: blessed
script {
use 0x0::Libra;
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::Transaction;

fun main(account: &signer) {
    Libra::burn<Coin1>(account, {{association}});
    Libra::burn<Coin2>(account, {{association}});
    Transaction::assert(Libra::market_cap<Coin1>() == 0, 9);
    Transaction::assert(Libra::market_cap<Coin2>() == 0, 10);
}
}
// check: EXECUTED

// check that stop minting works
//! new-transaction
//! sender: blessed
script {
use 0x0::Libra;
use 0x0::Coin1::Coin1;

fun main(account: &signer) {
    Libra::update_minting_ability<Coin1>(account, false);
    let coin = Libra::mint<Coin1>(account, 10); // will abort here
    Libra::destroy_zero(coin)
}
}
// check: ABORTED
// check: 4
