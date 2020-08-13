//! account: dd1, 0, 0, address
//! account: dd2, 0, 0, address

//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::Libra;

// register dd(1|2) as a preburner
fun main(account: &signer) {
    let prev_mcap1 = Libra::market_cap<Coin1>();
    let prev_mcap2 = Libra::market_cap<Coin2>();
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        {{dd1}},
        {{dd1::auth_key}},
        x"",
        false,
    );
    LibraAccount::create_designated_dealer<Coin2>(
        account,
        {{dd2}},
        {{dd2::auth_key}},
        x"",
        false,
    );
    LibraAccount::tiered_mint<Coin1>(
        account,
        {{dd1}},
        10,
        0,
    );
    LibraAccount::tiered_mint<Coin2>(
        account,
        {{dd2}},
        100,
        0,
    );
    assert(Libra::market_cap<Coin1>() - prev_mcap1 == 10, 7);
    assert(Libra::market_cap<Coin2>() - prev_mcap2 == 100, 8);
}
}
// check: EXECUTED

//! new-transaction
//! sender: dd1
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::preburn<Coin1>(account, &with_cap, 10);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: dd2
script {
use 0x1::Coin2::Coin2;
use 0x1::LibraAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::preburn<Coin2>(account, &with_cap, 100);
    LibraAccount::restore_withdraw_capability(with_cap);
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
    let prev_mcap1 = Libra::market_cap<Coin1>();
    let prev_mcap2 = Libra::market_cap<Coin2>();
    Libra::burn<Coin1>(account, {{dd1}});
    Libra::burn<Coin2>(account, {{dd2}});
    assert(prev_mcap1 - Libra::market_cap<Coin1>() == 10, 9);
    assert(prev_mcap2 - Libra::market_cap<Coin2>() == 100, 10);
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
    Libra::destroy_zero(coin);
}
}
// check: ABORTED
// check: 3
