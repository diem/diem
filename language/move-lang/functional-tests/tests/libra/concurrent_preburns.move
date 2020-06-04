// Test the concurrent preburn-burn flow

//! account: preburner, 0Coin1

// create 100 Coin1's for the preburner. we can't do this using the //! account macro because it
// doesn't increment the market cap appropriately

//! sender: association
//! max-gas: 1000000
//! gas-price: 0
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::LibraAccount;
fun main(account: &signer) {
    let coin = Libra::mint<Coin1>(account, 600);
    LibraAccount::deposit(account, {{preburner}}, coin);
}
}

// register the sender as a preburn entity
//! new-transaction
//! sender: preburner
//! max-gas: 1000000
//! gas-price: 0
//! gas-currency: Coin1
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
fun main(account: &signer) {
    Libra::publish_preburn(account, Libra::new_preburn<Coin1>())
}
}

// check: EXECUTED

// perform three preburns: 100, 200, 300
//! new-transaction
//! sender: preburner
//! max-gas: 1000000
//! gas-price: 0
//! gas-currency: Coin1
script {
use 0x0::LibraAccount;
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::Transaction;
fun main(account: &signer) {
    let coin100 = LibraAccount::withdraw_from<Coin1>(account, 100);
    let coin200 = LibraAccount::withdraw_from<Coin1>(account, 200);
    let coin300 = LibraAccount::withdraw_from<Coin1>(account, 300);
    Libra::preburn_to<Coin1>(account, coin100);
    Libra::preburn_to<Coin1>(account, coin200);
    Libra::preburn_to<Coin1>(account, coin300);
    Transaction::assert(Libra::preburn_value<Coin1>() == 600, 8001)
}
}

// check: PreburnEvent
// check: PreburnEvent
// check: PreburnEvent
// check: EXECUTED

// perform three burns. order should match the preburns
//! new-transaction
//! sender: blessed
//! max-gas: 1000000
//! gas-price: 0
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::Transaction;
fun main(account: &signer) {
    let burn_address = {{preburner}};
    Libra::burn<Coin1>(account, burn_address);
    Transaction::assert(Libra::preburn_value<Coin1>() == 500, 8002);
    Libra::burn<Coin1>(account, burn_address);
    Transaction::assert(Libra::preburn_value<Coin1>() == 300, 8003);
    Libra::burn<Coin1>(account, burn_address);
    Transaction::assert(Libra::preburn_value<Coin1>() == 0, 8004)
}
}

// check: BurnEvent
// check: BurnEvent
// check: BurnEvent
// check: EXECUTED
