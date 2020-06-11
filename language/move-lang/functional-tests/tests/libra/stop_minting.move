//! sender: association
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;

// register blessed as a preburner
fun main(account: &signer) {
    LibraAccount::add_preburn_from_association<Coin1>(account, {{blessed}});
    LibraAccount::add_preburn_from_association<Coin2>(account, {{blessed}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;

// do some preburning
fun main(account: &signer) {
    let coin1_coins = Libra::mint<Coin1>(account, 10);
    let coin2_coins = Libra::mint<Coin2>(account, 100);
    assert(Libra::market_cap<Coin1>() == 10, 7);
    assert(Libra::market_cap<Coin2>() == 100, 8);
    Libra::preburn_to(account, coin1_coins);
    Libra::preburn_to(account, coin2_coins);
}
}
// check: EXECUTED

// do some burning
//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;

fun main(account: &signer) {
    Libra::burn<Coin1>(account, {{blessed}});
    Libra::burn<Coin2>(account, {{blessed}});
    assert(Libra::market_cap<Coin1>() == 0, 9);
    assert(Libra::market_cap<Coin2>() == 0, 10);
}
}
// check: EXECUTED

// check that stop minting works
//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;

fun main(account: &signer) {
    Libra::update_minting_ability<Coin1>(account, false);
    let coin = Libra::mint<Coin1>(account, 10); // will abort here
    Libra::destroy_zero(coin)
}
}
// check: ABORTED
// check: 4
