//! account: bob, 100000LBR
//! account: alice, 100000Coin1
//! account: gary, 100000Coin1
//! account: vivian, 100000Coin2
//! account: nope, 100000Coin2

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
    PaymentRouter::allow_account_address({{gary}});
    PaymentRouter::allow_account_address({{vivian}});
    PaymentRouter::allow_currency<Coin1::T>();
    PaymentRouter::allow_currency<Coin2::T>();
    PaymentRouter::allow_currency<LBR::T>();
}
}

//! new-transaction
//! sender: alice
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{bob}});
}
}

//! new-transaction
//! sender: gary
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{bob}});
}
}

//! new-transaction
//! sender: vivian
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
use 0x0::Coin2;
fun main() {
    PaymentRouter::add_account_to<Coin2::T>({{bob}});
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
//! sender: nope
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
use 0x0::Coin2;
fun main() {
    PaymentRouter::add_account_to<Coin2::T>({{bob}});
}
}
// check: ABORTED
// check: 0

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 1000000
script {
use 0x0::Vector;
use 0x0::Transaction;
use 0x0::PaymentRouter;
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
fun main() {
    let addrs_coin1 = PaymentRouter::addresses_for_currency<Coin1::T>(Transaction::sender());
    let addrs_coin2 = PaymentRouter::addresses_for_currency<Coin2::T>(Transaction::sender());
    let addrs_lbr = PaymentRouter::addresses_for_currency<LBR::T>(Transaction::sender());

    Transaction::assert(Vector::length(&addrs_coin1) == 2, 0);
    Transaction::assert(Vector::length(&addrs_coin2) == 1, 1);
    Transaction::assert(Vector::length(&addrs_lbr) == 1, 2);
}
}

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 1000000
script {
use 0x0::Transaction;
use 0x0::PaymentRouter;
use 0x0::Coin1;
use 0x0::LibraAccount;
// Withdraw from the router "root" account.
fun main() {
    let prev_balance = LibraAccount::balance<Coin1::T>({{alice}});
    let x_coins = PaymentRouter::withdraw<Coin1::T>(10);
    let new_balance = LibraAccount::balance<Coin1::T>({{alice}});
    Transaction::assert(prev_balance - new_balance == 10, 0);
    PaymentRouter::deposit<Coin1::T>({{bob}}, x_coins);
    new_balance = LibraAccount::balance<Coin1::T>({{alice}});
    Transaction::assert(prev_balance - new_balance == 0, 1);
}
}

//! new-transaction
//! sender: alice
//! gas-price: 0
//! max-gas: 1000000
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
// Try to have alice withdraw through the payment router. But this doesn't
// work since `exclusive_withdrawals_only` is set to true.
fun main() {
    let x_coins = PaymentRouter::withdraw_through<Coin1::T>(10);
    PaymentRouter::deposit<Coin1::T>({{bob}}, x_coins);
}
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 1000000
script {
use 0x0::PaymentRouter;
use 0x0::LBR;
// Try to have bob withdraw through the payment router owned by bob. But this doesn't
// work since `exclusive_withdrawals_only` is set to true.
fun main() {
    let x_coins = PaymentRouter::withdraw_through<LBR::T>(10);
    PaymentRouter::deposit<LBR::T>({{bob}}, x_coins);
}
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 100000
script {
use 0x0::PaymentRouter;
fun main() {
    PaymentRouter::set_exclusive_withdrawals(false);
}
}

//! new-transaction
//! sender: alice
//! gas-price: 0
//! max-gas: 1000000
script {
use 0x0::Transaction;
use 0x0::PaymentRouter;
use 0x0::Coin1;
use 0x0::LibraAccount;
// Try to have alice withdraw through the payment router. This now succeeds
// since we set `exclusive_withdrawals_only` to false.
fun main() {
    let prev_balance = LibraAccount::balance<Coin1::T>({{alice}});
    let x_coins = PaymentRouter::withdraw_through<Coin1::T>(10);
    let new_balance = LibraAccount::balance<Coin1::T>({{alice}});
    Transaction::assert(prev_balance - new_balance == 10, 0);
    PaymentRouter::deposit<Coin1::T>({{bob}}, x_coins);
    new_balance = LibraAccount::balance<Coin1::T>({{alice}});
    Transaction::assert(prev_balance - new_balance == 0, 1);
}
}
