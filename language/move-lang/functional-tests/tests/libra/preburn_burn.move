//! account: dd, 0, 0, address
// Test the end-to-end preburn-burn flow

// register blessed as a preburn entity
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::DesignatedDealer;

fun main(account: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        {{dd}},
        {{dd::auth_key}},
        x"",
        x"",
        pubkey,
        false,
    );
    DesignatedDealer::add_tier(account, {{dd}}, 1000);
    LibraAccount::tiered_mint<Coin1>(
        account,
        {{dd}},
        600,
        0,
    );
}
}
// check: EXECUTED

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
// check: EXECUTED

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

// check: 6
// check: ABORTED



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
// check: EXECUTED

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

// check: 10
// check: ABORTED
