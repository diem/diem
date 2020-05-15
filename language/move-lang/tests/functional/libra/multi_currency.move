//! account: bob, 10Coin1
//! account: alice, 10Coin2

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
use 0x0::LibraAccount;
use 0x0::Coin2;
fun main() {
    LibraAccount::add_currency<Coin2::T>();
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-currency: Coin2
script {
use 0x0::LibraAccount;
use 0x0::Coin1;
fun main() {
    LibraAccount::add_currency<Coin1::T>();
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-currency: Coin2
script {
use 0x0::LibraAccount;
use 0x0::Coin2;
use 0x0::Transaction;
fun main() {
    LibraAccount::pay_from_sender<Coin2::T>({{bob}}, 10);
    Transaction::assert(LibraAccount::balance<Coin2::T>({{alice}}) == 0, 0);
    Transaction::assert(LibraAccount::balance<Coin2::T>({{bob}}) == 10, 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
use 0x0::LibraAccount;
use 0x0::Coin2;
use 0x0::Coin1;
use 0x0::Transaction;
fun main() {
    LibraAccount::pay_from_sender<Coin2::T>({{alice}}, 10);
    LibraAccount::pay_from_sender<Coin1::T>({{alice}}, 10);
    Transaction::assert(LibraAccount::balance<Coin1::T>({{bob}}) == 0, 2);
    Transaction::assert(LibraAccount::balance<Coin2::T>({{bob}}) == 0, 3);
    Transaction::assert(LibraAccount::balance<Coin1::T>({{alice}}) == 10, 4);
    Transaction::assert(LibraAccount::balance<Coin2::T>({{alice}}) == 10, 5);
}
}
// check: EXECUTED
