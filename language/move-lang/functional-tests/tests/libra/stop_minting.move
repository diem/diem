//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::Roles::{Self, TreasuryComplianceRole};

// register blessed as a preburner
fun main(account: &signer) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
    LibraAccount::add_preburn_from_tc<Coin1>(&tc_capability, {{blessed}});
    LibraAccount::add_preburn_from_tc<Coin2>(&tc_capability, {{blessed}});
    Roles::restore_capability_to_privilege(account, tc_capability);
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
use 0x1::Roles::{Self, TreasuryComplianceRole};

fun main(account: &signer) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
    Libra::update_minting_ability<Coin1>(&tc_capability, false);
    let coin = Libra::mint<Coin1>(account, 10); // will abort here
    Libra::destroy_zero(coin);
    Roles::restore_capability_to_privilege(account, tc_capability);
}
}
// check: ABORTED
// check: 4
