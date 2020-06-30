//! account: dd, 0, 0, address
// Test the concurrent preburn-burn flow

// register blessed as a preburn entity
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Roles::{Self, TreasuryComplianceRole};
use 0x1::DesignatedDealer;

// register dd as a preburner
fun main(account: &signer) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        &tc_capability,
        {{dd}},
        {{dd::auth_key}},
    );
    let coins = DesignatedDealer::tiered_mint<Coin1>(
        account,
        &tc_capability,
        600,
        {{dd}},
        0,
    );
    LibraAccount::deposit(account, {{dd}}, coins);
    Roles::restore_capability_to_privilege(account, tc_capability);
}
}
// check: EXECUTED

// perform three preburns: 100, 200, 300
//! new-transaction
//! sender: dd
//! gas-currency: Coin1
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin100 = LibraAccount::withdraw_from<Coin1>(&with_cap, 100);
    let coin200 = LibraAccount::withdraw_from<Coin1>(&with_cap, 200);
    let coin300 = LibraAccount::withdraw_from<Coin1>(&with_cap, 300);
    Libra::preburn_to<Coin1>(account, coin100);
    Libra::preburn_to<Coin1>(account, coin200);
    Libra::preburn_to<Coin1>(account, coin300);
    assert(Libra::preburn_value<Coin1>() == 600, 8001);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}

// check: PreburnEvent
// check: PreburnEvent
// check: PreburnEvent
// check: EXECUTED

// perform three burns. order should match the preburns
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let burn_address = {{dd}};
    Libra::burn<Coin1>(account, burn_address);
    assert(Libra::preburn_value<Coin1>() == 500, 8002);
    Libra::burn<Coin1>(account, burn_address);
    assert(Libra::preburn_value<Coin1>() == 300, 8003);
    Libra::burn<Coin1>(account, burn_address);
    assert(Libra::preburn_value<Coin1>() == 0, 8004)
}
}

// check: BurnEvent
// check: BurnEvent
// check: BurnEvent
// check: EXECUTED
