// Test the end-to-end preburn-burn flow

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

// perform a preburn
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let coin = Libra::mint<Coin1>(account, 100);
    let old_market_cap = Libra::market_cap<Coin1>();
    // send the coins to the preburn bucket. market cap should not be affected, but the preburn
    // bucket should increase in size by 100
    Libra::preburn_to<Coin1>(account, coin);
    assert(Libra::market_cap<Coin1>() == old_market_cap, 8002);
    assert(Libra::preburn_value<Coin1>() == 100, 8003);
}
}

// check: PreburnEvent
// check: EXECUTED

// perform the burn from the blessed account
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let old_market_cap = Libra::market_cap<Coin1>();
    // do the burn. the market cap should now decrease, and the preburn bucket should be empty
    Libra::burn<Coin1>(account, {{blessed}});
    assert(Libra::market_cap<Coin1>() == old_market_cap - 100, 8004);
    assert(Libra::preburn_value<Coin1>() == 0, 8005);
}
}

// check: BurnEvent
// check: EXECUTED
