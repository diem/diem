//! new-transaction
//! sender: association
script {
use 0x0::Libra;
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::Transaction;
// Make sure that Coin1 and Coin2 are registered
fun main() {
    Transaction::assert(Libra::is_currency<Coin1>(), 1);
    Transaction::assert(Libra::is_currency<Coin2>(), 2);
    Transaction::assert(!Libra::is_synthetic_currency<Coin1>(), 2);
    Transaction::assert(!Libra::is_synthetic_currency<Coin2>(), 3);
    Transaction::assert(Libra::market_cap<Coin1>() == 0, 4);
    Transaction::assert(Libra::market_cap<Coin2>() == 0, 5);
}
}
// check: EXECUTED
