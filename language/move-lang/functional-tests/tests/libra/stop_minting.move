//! new-transaction
//! sender: blessed
script {
use 0x0::Libra;
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::Signer;
use 0x0::Transaction;

// Make sure we can mint and burn
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    let pre_coin1 = Libra::new_preburn<Coin1>();
    let pre_coin2 = Libra::new_preburn<Coin2>();
    Libra::publish_preburn(account, pre_coin1);
    Libra::publish_preburn(account, pre_coin2);
    let coin1_coins = Libra::mint<Coin1>(account, 10);
    let coin2_coins = Libra::mint<Coin2>(account, 10);
    Transaction::assert(Libra::market_cap<Coin1>() == 10, 7);
    Transaction::assert(Libra::market_cap<Coin2>() == 10, 8);
    Libra::preburn_to(account, coin1_coins);
    Libra::preburn_to(account, coin2_coins);
    Libra::burn<Coin1>(account, sender);
    Libra::burn<Coin2>(account, sender);
    Transaction::assert(Libra::market_cap<Coin1>() == 0, 9);
    Transaction::assert(Libra::market_cap<Coin2>() == 0, 10);

    let coin1_coins = Libra::mint<Coin1>(account, 10);
    let coin2_coins = Libra::mint<Coin2>(account, 10);

    Libra::update_minting_ability<Coin1>(account, false);
    Libra::preburn_to(account, coin1_coins);
    Libra::preburn_to(account, coin2_coins);
    Libra::burn<Coin1>(account, sender);
    Libra::burn<Coin2>(account, sender);
    Transaction::assert(Libra::market_cap<Coin1>() == 0, 11);
    Transaction::assert(Libra::market_cap<Coin2>() == 0, 12);
    Libra::preburn_to(account, Libra::mint<Coin2>(account, 10));
    Libra::preburn_to(account, Libra::mint<Coin1>(account, 10))
}
}
// check: ABORTED
// check: 4
