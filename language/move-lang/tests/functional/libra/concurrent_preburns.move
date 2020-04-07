// Test the concurrent preburn-burn flow with the simplest possible scenario: burner and preburner
// are the same entity.

// register the sender as a preburn entity
//! sender: association
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    Libra::publish_preburn(Libra::new_preburn<LBR::T>())
}

// check: EXECUTED

// perform three preburns: 100, 200, 300
//! new-transaction
//! sender: association
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let coin100 = LibraAccount::withdraw_from_sender<LBR::T>(100);
    let coin200 = LibraAccount::withdraw_from_sender<LBR::T>(200);
    let coin300 = LibraAccount::withdraw_from_sender<LBR::T>(300);
    Libra::preburn_to_sender<LBR::T>(coin100);
    Libra::preburn_to_sender<LBR::T>(coin200);
    Libra::preburn_to_sender<LBR::T>(coin300);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 600, 8001)
}

// check: EXECUTED

// perform three burns. order should match the preburns
//! new-transaction
//! sender: association
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let burn_address = {{association}};
    Libra::burn<LBR::T>(burn_address);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 500, 8002);
    Libra::burn<LBR::T>(burn_address);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 300, 8003);
    Libra::burn<LBR::T>(burn_address);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 0, 8004)
}

// check: EXECUTED
