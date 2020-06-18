//! account: dd1, 0, 0, address
//! account: dd2, 0, 0, address

//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::Roles::{Self, TreasuryComplianceRole};
use 0x1::DesignatedDealer;
use 0x1::Libra;

// register dd(1|2) as a preburner
fun main(account: &signer) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        &tc_capability,
        {{dd1}},
        {{dd1::auth_key}},
    );
    LibraAccount::create_designated_dealer<Coin2>(
        account,
        &tc_capability,
        {{dd2}},
        {{dd2::auth_key}},
    );
    let coin1 = DesignatedDealer::tiered_mint<Coin1>(
        account,
        &tc_capability,
        10,
        {{dd1}},
        0,
    );
    let coin2 = DesignatedDealer::tiered_mint<Coin2>(
        account,
        &tc_capability,
        100,
        {{dd2}},
        0,
    );
    assert(Libra::market_cap<Coin1>() == 10, 7);
    assert(Libra::market_cap<Coin2>() == 100, 8);
    LibraAccount::deposit(account, {{dd1}}, coin1);
    LibraAccount::deposit(account, {{dd2}}, coin2);
    Roles::restore_capability_to_privilege(account, tc_capability);
}
}
// check: EXECUTED

//! new-transaction
//! sender: dd1
//! gas-currency: Coin1
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin1_coins = LibraAccount::withdraw_from<Coin1>(&with_cap, 10);
    Libra::preburn_to(account, coin1_coins);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: dd2
//! gas-currency: Coin2
script {
use 0x1::Libra;
use 0x1::Coin2::Coin2;
use 0x1::LibraAccount;

// do some preburning
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin2_coins = LibraAccount::withdraw_from<Coin2>(&with_cap, 100);
    Libra::preburn_to(account, coin2_coins);
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
    Libra::burn<Coin1>(account, {{dd1}});
    Libra::burn<Coin2>(account, {{dd2}});
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
