// Test the mint flow

//! account: alice, 0Coin1

// Minting from a privileged account should work
//! sender: association
use 0x0::Coin1;
use 0x0::Libra;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main() {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Libra::market_cap<Coin1::T>();
    let coin = Libra::mint<Coin1::T>(100);
    Transaction::assert(Libra::value<Coin1::T>(&coin) == 100, 8000);
    Transaction::assert(Libra::market_cap<Coin1::T>() == old_market_cap + 100, 8001);

    // get rid of the coin
    LibraAccount::deposit({{alice}}, coin);
}

// check: MintEvent
// check: EXECUTED

//! new-transaction
// Minting from a non-privileged account should not work
use 0x0::Coin1;
use 0x0::Libra;
use 0x0::LibraAccount;
fun main() {
    let coin = Libra::mint<Coin1::T>(100);
    LibraAccount::deposit_to_sender<Coin1::T>(coin)
}

// will fail with MISSING_DATA because sender doesn't have the mint capability
// check: Keep
// check: MISSING_DATA
