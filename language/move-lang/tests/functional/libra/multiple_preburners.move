// Test the end-to-end preburn-burn flow with two different preburners (alice and bob) that are
// distinct from the burner (the association). Use a third, non-privileged account (carol) to check
// that only preburners can preburn and only the associaton can burn.

//! account: alice
//! account: bob
//! account: carol

// publish preburn resource to alice's account
//! sender: alice
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    Libra::publish_preburn(Libra::new_preburn<LBR::T>())
}

// check: EXECUTED

// perform a preburn of 100 from alice's account
//! new-transaction
//! sender: alice
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(100);
    Libra::preburn_to_sender<LBR::T>(coin);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 100, 8001)
}

// check: EXECUTED

// preburn resource to bob's account
//! new-transaction
//! sender: bob
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    Libra::publish_preburn(Libra::new_preburn<LBR::T>())
}

// check: EXECUTED

// perform a preburn of 200 from bob's account
//! new-transaction
//! sender: bob
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(200);
    Libra::preburn_to_sender<LBR::T>(coin);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 300, 8002)
}

// check: EXECUTED

// ensure that the non-privileged user carol cannot preburn.
// will fail with MISSING_DATA because sender doesn't have a Preburn resource
//! new-transaction
//! sender: carol
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(200);
    Libra::preburn_to_sender<LBR::T>(coin);
}

// check: Keep
// check: MISSING_DATA

// ensure that the non-privileged user carol cannot burn.
// will fail with MISSING_DATA because sender doesn't have the mint capability
//! new-transaction
//! sender: carol
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    Libra::burn<LBR::T>({{bob}})
}

// check: Keep
// check: MISSING_DATA

// ensure that the preburner bob cannot burn
//! new-transaction
//! sender: bob
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    Libra::burn<LBR::T>({{bob}})
}

// check: Keep
// check: MISSING_DATA

// burn bob's funds, then alice's
//! new-transaction
//! sender: association
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    Libra::burn<LBR::T>({{bob}});
    Transaction::assert(Libra::preburn_value<LBR::T>() == 100, 8003);
    Libra::burn<LBR::T>({{alice}});
    Transaction::assert(Libra::preburn_value<LBR::T>() == 0, 8004)
}

// check: EXECUTED

// now, we will initiate a burn request from alice and have the association cancel/return it.
//! new-transaction
//! sender: alice
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(100);
    Libra::preburn_to_sender<LBR::T>(coin);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 100, 8005)
}

// check: EXECUTED

// cancel Alice's request and return her funds
//! new-transaction
//! sender: association
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let old_balance = LibraAccount::balance<LBR::T>({{alice}});
    LibraAccount::cancel_burn<LBR::T>({{alice}});
    Transaction::assert(Libra::preburn_value<LBR::T>() == 0, 8006);
    Transaction::assert(LibraAccount::balance<LBR::T>({{alice}}) == old_balance + 100, 8007)
}

// check: EXECUTED
