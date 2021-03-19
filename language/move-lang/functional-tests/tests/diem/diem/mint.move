// Test the mint flow

//! account: alice, 0XUS

module Holder {
    struct Holder<T> has key { x: T }
    public fun hold<T: store>(account: &signer, x: T)  {
        move_to(account, Holder<T> { x })
    }
}
// check: "Keep(EXECUTED)"

// Minting from a privileged account should work
//! new-transaction
//! sender: blessed
script {
use 0x1::XUS::XUS;
use 0x1::Diem;
use {{default}}::Holder;
fun main(account: signer) {
    let account = &account;
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Diem::market_cap<XUS>();
    let coin = Diem::mint<XUS>(account, 100);
    assert(Diem::value<XUS>(&coin) == 100, 8000);
    assert(Diem::market_cap<XUS>() == old_market_cap + 100, 8001);

    // get rid of the coin
    Holder::hold(account, coin)
}
}

// check: MintEvent
// check: "Keep(EXECUTED)"

//! new-transaction
// Minting from a non-privileged account should not work
script {
use 0x1::XUS::XUS;
use 0x1::Diem;
fun main(account: signer) {
    let account = &account;
    let coin = Diem::mint<XUS>(account, 100);
    Diem::destroy_zero(coin)
}
}

// will abort because sender doesn't have the mint capability
// check: "Keep(ABORTED { code: 2308,"
