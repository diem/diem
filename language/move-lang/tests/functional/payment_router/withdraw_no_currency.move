//! account: bob, 100000LBR
//! account: alice, 100000Coin1
//! account: vivian, 100000Coin2

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
fun main() {
    PaymentRouter::initialize(true);
    PaymentRouter::allow_account_address({{bob}});
    PaymentRouter::allow_account_address({{alice}});
    PaymentRouter::allow_currency<Coin1::T>();
    PaymentRouter::allow_currency<Coin2::T>();
    PaymentRouter::allow_currency<LBR::T>();
}
}

//! new-transaction
//! sender: alice
//! gas-price: 0
//! max-gas: 100000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{bob}});
}
}

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
use 0x0::LBR;
fun main() {
    PaymentRouter::add_account_to<LBR::T>({{bob}});
}
}

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 1000000
script {
use 0x0::PaymentRouter;
use 0x0::Coin2;
fun main() {
    let x_coins = PaymentRouter::withdraw<Coin2::T>(10);
    PaymentRouter::deposit<Coin2::T>({{bob}}, x_coins);
}
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: vivian
//! gas-price: 0
//! max-gas: 1000000
//! gas-currency: Coin2
script {
use 0x0::PaymentRouter;
use 0x0::Coin2;
use 0x0::LibraAccount;
fun main() {
    let x_coins = LibraAccount::withdraw_from_sender<Coin2::T>(10);
    PaymentRouter::deposit<Coin2::T>({{bob}}, x_coins);
}
}
// check: ABORTED
// check: 1
