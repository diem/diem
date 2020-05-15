//! account: bob, 100000Coin1
//! account: alice, 100000Coin1
//! account: gary, 100000Coin1
//! account: vivian, 100000Coin1
//! account: nope, 100000Coin1

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 100000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::initialize(true);
    PaymentRouter::allow_account_address({{bob}});
    PaymentRouter::allow_account_address({{gary}});
    PaymentRouter::allow_account_address({{nope}});
    PaymentRouter::allow_currency<Coin1::T>();
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
    PaymentRouter::initialize(false);
    PaymentRouter::allow_account_address({{alice}});
    PaymentRouter::allow_account_address({{vivian}});
    // nope could be added to both.
    PaymentRouter::allow_account_address({{nope}});
    PaymentRouter::allow_currency<Coin1::T>();
}
}

//! new-transaction
//! sender: gary
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
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{bob}});
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
    PaymentRouter::add_account_to<Coin1::T>({{alice}});
}
}

//! new-transaction
//! sender: vivian
//! gas-price: 0
//! max-gas: 100000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{alice}});
}
}

//! new-transaction
//! sender: gary
//! gas-price: 0
//! max-gas: 1000000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
// Try to have gary withdraw through bob's payment router. But this doesn't
// work since bob has set the exclusive_withdrawal_flag to true.
fun main() {
    let x_coins = PaymentRouter::withdraw_through<Coin1::T>(10);
    PaymentRouter::deposit<Coin1::T>({{bob}}, x_coins);
}
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: vivian
//! gas-price: 0
//! max-gas: 1000000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
// vivan can withdraw through alice's payment router since alice has set
// the exclusive_withdrawal_flag to false.
fun main() {
    let x_coins = PaymentRouter::withdraw_through<Coin1::T>(10);
    PaymentRouter::deposit<Coin1::T>({{alice}}, x_coins);
}
}

//! new-transaction
//! sender: vivian
//! gas-price: 0
//! max-gas: 1000000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
use 0x0::Transaction;
fun main() {
    Transaction::assert(PaymentRouter::is_routed<Coin1::T>({{bob}}), 0);
    Transaction::assert(PaymentRouter::is_routed<Coin1::T>({{gary}}), 0);
    Transaction::assert(PaymentRouter::is_routed<Coin1::T>({{alice}}), 0);
    Transaction::assert(PaymentRouter::is_routed<Coin1::T>({{vivian}}), 0);
    Transaction::assert(!PaymentRouter::is_routed<Coin1::T>({{nope}}), 1);

    Transaction::assert(PaymentRouter::router_address<Coin1::T>({{bob}}) == {{bob}}, 2);
    Transaction::assert(PaymentRouter::router_address<Coin1::T>({{gary}}) == {{bob}}, 2);
    Transaction::assert(PaymentRouter::router_address<Coin1::T>({{alice}}) == {{alice}}, 2);
    Transaction::assert(PaymentRouter::router_address<Coin1::T>({{vivian}}) == {{alice}}, 2);
}
}

//! new-transaction
//! sender: nope
//! gas-price: 0
//! max-gas: 100000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{alice}});
}
}

//! new-transaction
//! sender: nope
//! gas-price: 0
//! max-gas: 100000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
// an account can't belong to multiple routers. This fails since nope is
// already routed by the payment router at alice.
fun main() {
    PaymentRouter::add_account_to<Coin1::T>({{bob}});
}
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: nope
//! gas-price: 0
//! max-gas: 100000
//! gas-currency: Coin1
script {
use 0x0::PaymentRouter;
use 0x0::Coin1;
use 0x0::Vector;
use 0x0::Transaction;
fun main() {
    let bobs = PaymentRouter::addresses_for_currency<Coin1::T>({{bob}});
    let alices = PaymentRouter::addresses_for_currency<Coin1::T>({{alice}});

    // alices addresses for Coin1 should have nope, but bob's should not
    Transaction::assert(!Vector::contains(&bobs, &{{nope}}), 3);
    Transaction::assert(Vector::contains(&alices, &{{nope}}), 4);
}
}
