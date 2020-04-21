// Test the concurrent preburn-burn flow

//! account: preburner, 0Coin1

// create 100 Coin1's for the preburner. we can't do this using the //! account macro because it
// doesn't increment the market cap appropriately

//! sender: association
//! max-gas: 1000000
//! gas-price: 0
use 0x0::Coin1;
use 0x0::Libra;
use 0x0::LibraAccount;
fun main() {
    let coin = Libra::mint<Coin1::T>(600);
    LibraAccount::deposit({{preburner}}, coin);
}

// register the sender as a preburn entity
//! new-transaction
//! sender: preburner
//! max-gas: 1000000
//! gas-price: 0
use 0x0::Coin1;
use 0x0::Libra;
fun main() {
    Libra::publish_preburn(Libra::new_preburn<Coin1::T>())
}

// check: EXECUTED

// perform three preburns: 100, 200, 300
//! new-transaction
//! sender: preburner
//! max-gas: 1000000
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::Coin1;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let coin100 = LibraAccount::withdraw_from_sender<Coin1::T>(100);
    let coin200 = LibraAccount::withdraw_from_sender<Coin1::T>(200);
    let coin300 = LibraAccount::withdraw_from_sender<Coin1::T>(300);
    Libra::preburn_to_sender<Coin1::T>(coin100);
    Libra::preburn_to_sender<Coin1::T>(coin200);
    Libra::preburn_to_sender<Coin1::T>(coin300);
    Transaction::assert(Libra::preburn_value<Coin1::T>() == 600, 8001)
}

// check: PreburnEvent
// check: PreburnEvent
// check: PreburnEvent
// check: EXECUTED

// perform three burns. order should match the preburns
//! new-transaction
//! sender: association
//! max-gas: 1000000
//! gas-price: 0
use 0x0::Coin1;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let burn_address = {{preburner}};
    Libra::burn<Coin1::T>(burn_address);
    Transaction::assert(Libra::preburn_value<Coin1::T>() == 500, 8002);
    Libra::burn<Coin1::T>(burn_address);
    Transaction::assert(Libra::preburn_value<Coin1::T>() == 300, 8003);
    Libra::burn<Coin1::T>(burn_address);
    Transaction::assert(Libra::preburn_value<Coin1::T>() == 0, 8004)
}

// check: BurnEvent
// check: BurnEvent
// check: BurnEvent
// check: EXECUTED
