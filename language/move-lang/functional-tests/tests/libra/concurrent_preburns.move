// Test the concurrent preburn-burn flow

// register blessed as a preburn entity
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Roles::{Self, TreasuryComplianceRole};

// register blessed as a preburner
fun main(account: &signer) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
    LibraAccount::add_preburn_from_tc<Coin1>(&tc_capability, {{blessed}});
    Roles::restore_capability_to_privilege(account, tc_capability);
}
}
// check: EXECUTED

// perform three preburns: 100, 200, 300
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let coin100 = Libra::mint<Coin1>(account, 100);
    let coin200 = Libra::mint<Coin1>(account, 200);
    let coin300 = Libra::mint<Coin1>(account, 300);
    Libra::preburn_to<Coin1>(account, coin100);
    Libra::preburn_to<Coin1>(account, coin200);
    Libra::preburn_to<Coin1>(account, coin300);
    assert(Libra::preburn_value<Coin1>() == 600, 8001)
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
    let burn_address = {{blessed}};
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
