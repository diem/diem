// Test the mint flow

//! account: alice, 0Coin1

module Holder {
    resource struct Holder<T> { x: T }
    public fun hold<T>(account: &signer, x: T)  {
        move_to(account, Holder<T> { x })
    }
}
// check: "Keep(EXECUTED)"

// Minting from a privileged account should work
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
use {{default}}::Holder;
fun main(account: &signer) {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Libra::market_cap<Coin1>();
    let coin = Libra::mint<Coin1>(account, 100);
    assert(Libra::value<Coin1>(&coin) == 100, 8000);
    assert(Libra::market_cap<Coin1>() == old_market_cap + 100, 8001);

    // get rid of the coin
    Holder::hold(account, coin)
}
}

// check: MintEvent
// check: "Keep(EXECUTED)"

//! new-transaction
// Minting from a non-privileged account should not work
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let coin = Libra::mint<Coin1>(account, 100);
    Libra::destroy_zero(coin)
}
}

// will abort because sender doesn't have the mint capability
// check: "Keep(ABORTED { code: 2308,"
