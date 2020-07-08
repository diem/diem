//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
// Make sure that Coin1 and Coin2 are registered
fun main() {
    assert(Libra::is_currency<Coin1>(), 1);
    assert(Libra::is_currency<Coin2>(), 2);
    assert(!Libra::is_synthetic_currency<Coin1>(), 2);
    assert(!Libra::is_synthetic_currency<Coin2>(), 3);
}
}
// check: EXECUTED
