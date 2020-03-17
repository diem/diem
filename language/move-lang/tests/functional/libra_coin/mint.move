// Test the mint flow

// Minting from a privileged account should work
//! sender: association
use 0x0::LibraCoin;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main() {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = LibraCoin::market_cap();
    let coin = LibraCoin::mint(100);
    Transaction::assert(LibraCoin::value(&coin) == 100, 8000);
    Transaction::assert(LibraCoin::market_cap() == old_market_cap + 100, 8001);

    LibraAccount::deposit_to_sender(coin)
}

// check: EXECUTED

//! new-transaction
// Minting from a non-privileged account should not work
use 0x0::LibraCoin;
use 0x0::LibraAccount;
fun main() {
    let coin = LibraCoin::mint(100);
    LibraAccount::deposit_to_sender(coin)
}

// will fail with MISSING_DATA because sender doesn't have the mint capability
// check: Keep
// check: MISSING_DATA
