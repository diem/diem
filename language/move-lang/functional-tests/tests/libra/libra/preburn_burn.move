//! account: dd, 0, 0, address
//! account: baddd, 0, 0, address
// Test the end-to-end preburn-burn flow

// register blessed as a preburn entity
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;

fun main(account: &signer) {
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        {{dd}},
        {{dd::auth_key}},
        x"",
        false,
    );
    LibraAccount::tiered_mint<Coin1>(
        account,
        {{dd}},
        600,
        0,
    );
}
}
// check: "Keep(EXECUTED)"

// perform a preburn
//! new-transaction
//! sender: dd
//! gas-currency: Coin1
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let old_market_cap = Libra::market_cap<Coin1>();
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    // send the coins to the preburn area. market cap should not be affected, but the preburn
    // bucket should increase in size by 100
    LibraAccount::preburn<Coin1>(account, &with_cap, 100);
    assert(Libra::market_cap<Coin1>() == old_market_cap, 8002);
    assert(Libra::preburn_value<Coin1>() == 100, 8003);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}

// check: PreburnEvent
// check: "Keep(EXECUTED)"

// cancel the preburn
//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer)  {
    LibraAccount::cancel_burn<Coin1>(account, {{dd}})
}
}
// check: CancelBurnEvent
// check: "Keep(EXECUTED)"

// perform a preburn
//! new-transaction
//! sender: dd
//! gas-currency: Coin1
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let old_market_cap = Libra::market_cap<Coin1>();
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    // send the coins to the preburn area. market cap should not be affected, but the preburn
    // bucket should increase in size by 100
    LibraAccount::preburn<Coin1>(account, &with_cap, 100);
    assert(Libra::market_cap<Coin1>() == old_market_cap, 8002);
    assert(Libra::preburn_value<Coin1>() == 100, 8003);
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: PreburnEvent
// check: "Keep(EXECUTED)"

// second (concurrent) preburn disallowed
//! new-transaction
//! sender: dd
//! gas-currency: Coin1
script {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        // Preburn area already occupied, aborts
        LibraAccount::preburn<Coin1>(account, &with_cap, 200);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 769,"

// perform the burn from the blessed account
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let old_market_cap = Libra::market_cap<Coin1>();
    // do the burn. the market cap should now decrease, and the preburn area should be empty
    Libra::burn<Coin1>(account, {{dd}});
    assert(Libra::market_cap<Coin1>() == old_market_cap - 100, 8004);
    assert(Libra::preburn_value<Coin1>() == 0, 8005);
    }
}

// check: BurnEvent
// check: "Keep(EXECUTED)"

// Preburn allowed but larger than balance
//! new-transaction
//! sender: dd
//! gas-currency: Coin1
script {
    use 0x1::Coin1::Coin1;
    // use 0x1::Libra;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::preburn<Coin1>(account, &with_cap, 501);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 1288,"

// Try to burn on an account that doesn't have a preburn resource
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    Libra::burn<Coin1>(account, {{default}});
}
}
// check: "Keep(ABORTED { code: 517,"

// Try to burn on an account that doesn't have a burn capability
//! new-transaction
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    Libra::burn<Coin1>(account, {{default}});
}
}
// check: "Keep(ABORTED { code: 4,"

// Try to cancel burn on an account that doesn't have a burn capability
//! new-transaction
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    Libra::destroy_zero(Libra::cancel_burn<Coin1>(account, {{dd}}));
}
}
// check: "Keep(ABORTED { code: 4,"

// Try to preburn to an account that doesn't have a preburn resource
//! new-transaction
script {
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    let coin = Libra::zero<Coin1>();
    Libra::preburn_to<Coin1>(account, coin)
}
}
// check: "Keep(ABORTED { code: 517,"

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
fun main(account: &signer) {
    LibraAccount::create_designated_dealer<LBR>(
        account,
        {{baddd}},
        {{baddd::auth_key}},
        x"",
        false,
    );
}
}
// check: "Keep(ABORTED { code: 1543,"

//! new-transaction
module Holder {
    resource struct Holder<T> {
        a: T,
        b: T,
    }

    public fun hold<T>(account: &signer, a: T, b: T) {
        move_to(account, Holder<T>{ a, b})
    }

    public fun get<T>(addr: address): (T, T)
    acquires Holder {
        let Holder { a, b} = move_from<Holder<T>>(addr);
        (a, b)
    }
}

// Limit exceeded on coin deposit
//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use {{default}}::Holder;
fun main(account: &signer) {
    let u64_max = 18446744073709551615;
    Holder::hold(
        account,
        Libra::mint<Coin1>(account, u64_max),
        Libra::mint<Coin1>(account, u64_max)
    );
}
}
// check: "Keep(EXECUTED)"

// Limit exceeded on coin deposit
//! new-transaction
//! sender: dd
script {
use 0x1::Libra::{Self, Libra};
use 0x1::Coin1::Coin1;
use {{default}}::Holder;
fun main(account: &signer) {
    let (coin1_tmp, coin2) = Holder::get<Libra<Coin1>>({{blessed}});
    Libra::preburn_to(account, coin1_tmp);
    Libra::preburn_to(account, coin2);
}
}
// check: "Keep(ABORTED { code: 769,"

// Limit exceeded on coin deposit
//! new-transaction
//! sender: dd
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let coin = Libra::zero<Coin1>();
    Libra::preburn_to(account, coin);
}
}
// check: "Keep(EXECUTED)"

// Limit exceeded on coin deposit
//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    Libra::burn<Coin1>(account, {{dd}});
}
}
// check: "Keep(ABORTED { code: 1025,"

//! new-transaction
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    Libra::publish_burn_capability(
        account,
        Libra::remove_burn_capability<Coin1>(account)
    );
}
}
// check: "Keep(ABORTED { code: 4,"
