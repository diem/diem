// Test the end-to-end preburn-burn flow with two different preburners (alice and bob) that are
// distinct from the burner (the association). Use a third, non-privileged account (carol) to check
// that only preburners can preburn and only the associaton can burn.

//! account: alice
//! account: bob
//! account: carol

// offer preburn resource to alice
//! sender: association
use 0x0::LibraCoin;
use 0x0::Offer;
fun main() {
    let preburn = LibraCoin::new_preburn();
    Offer::create(preburn, {{alice}})
}

// check: EXECUTED

// claim alice's preburn resource and perform a preburn of 100
//! new-transaction
//! sender: alice
use 0x0::LibraAccount;
use 0x0::LibraCoin;
use 0x0::Offer;
use 0x0::Transaction;
fun main() {
    LibraCoin::publish_preburn(Offer::redeem({{association}}));
    let coin = LibraAccount::withdraw_from_sender(100);
    LibraCoin::preburn(coin);
    Transaction::assert(LibraCoin::preburn_value() == 100, 8001)
}

// check: EXECUTED

// offer preburn resource to bob
//! new-transaction
//! sender: association
use 0x0::LibraCoin;
use 0x0::Offer;
fun main() {
    let preburn = LibraCoin::new_preburn();
    Offer::create(preburn, {{bob}})
}

// check: EXECUTED

// claim bob's preburn resource and perform a preburn of 200
//! new-transaction
//! sender: bob
use 0x0::LibraAccount;
use 0x0::LibraCoin;
use 0x0::Offer;
use 0x0::Transaction;
fun main() {
    LibraCoin::publish_preburn(Offer::redeem({{association}}));
    let coin = LibraAccount::withdraw_from_sender(200);
    LibraCoin::preburn(coin);
    Transaction::assert(LibraCoin::preburn_value() == 300, 8002)
}

// check: EXECUTED

// ensure that the non-privileged user carol cannot preburn.
// will fail with MISSING_DATA because sender doesn't have a Preburn resource
//! new-transaction
//! sender: carol
use 0x0::LibraAccount;
use 0x0::LibraCoin;
fun main() {
    let coin = LibraAccount::withdraw_from_sender(200);
    LibraCoin::preburn(coin);
}

// check: Keep
// check: MISSING_DATA

// ensure that the non-privileged user carol cannot burn.
// will fail with MISSING_DATA because sender doesn't have the mint capability
//! new-transaction
//! sender: carol
use 0x0::LibraCoin;
fun main() {
    LibraCoin::burn({{bob}})
}

// check: Keep
// check: MISSING_DATA

// ensure that the preburner bob cannot burn
//! new-transaction
//! sender: bob
use 0x0::LibraCoin;
fun main() {
    LibraCoin::burn({{bob}})
}

// check: Keep
// check: MISSING_DATA

// burn bob's funds, then alice's
//! new-transaction
//! sender: association
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    LibraCoin::burn({{bob}});
    Transaction::assert(LibraCoin::preburn_value() == 100, 8003);
    LibraCoin::burn({{alice}});
    Transaction::assert(LibraCoin::preburn_value() == 0, 8004)
}

// check: EXECUTED

// now, we will initiate a burn request from alice and have the association cancel/return it.
//! new-transaction
//! sender: alice
use 0x0::LibraAccount;
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    let coin = LibraAccount::withdraw_from_sender(100);
    LibraCoin::preburn(coin);
    Transaction::assert(LibraCoin::preburn_value() == 100, 8005)
}

// check: EXECUTED

// cancel Alice's request and return her funds
//! new-transaction
//! sender: association
use 0x0::LibraAccount;
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    let old_balance = LibraAccount::balance({{alice}});
    LibraAccount::cancel_burn({{alice}});
    Transaction::assert(LibraCoin::preburn_value() == 0, 8006);
    Transaction::assert(LibraAccount::balance({{alice}}) == old_balance + 100, 8007)
}

// check: EXECUTED
