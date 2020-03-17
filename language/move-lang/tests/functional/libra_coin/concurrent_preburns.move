// Test the concurrent preburn-burn flow with the simplest possible scenario: burner and preburner
// are the same entity.

// register the sender as a preburn entity
//! sender: association
use 0x0::LibraCoin;
fun main() {
    let preburn = LibraCoin::new_preburn();
    LibraCoin::publish_preburn(preburn)
}

// check: EXECUTED

// perform three preburns: 100, 200, 300
//! new-transaction
//! sender: association
use 0x0::LibraAccount;
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    let coin100 = LibraAccount::withdraw_from_sender(100);
    let coin200 = LibraAccount::withdraw_from_sender(200);
    let coin300 = LibraAccount::withdraw_from_sender(300);
    LibraCoin::preburn(coin100);
    LibraCoin::preburn(coin200);
    LibraCoin::preburn(coin300);
    Transaction::assert(LibraCoin::preburn_value() == 600, 8001)
}

// check: EXECUTED

// perform three burns. order should match the preburns
//! new-transaction
//! sender: association
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    let burn_address = {{association}};
    LibraCoin::burn(burn_address);
    Transaction::assert(LibraCoin::preburn_value() == 500, 8002);
    LibraCoin::burn(burn_address);
    Transaction::assert(LibraCoin::preburn_value() == 300, 8003);
    LibraCoin::burn(burn_address);
    Transaction::assert(LibraCoin::preburn_value() == 0, 8004)
}

// check: EXECUTED
