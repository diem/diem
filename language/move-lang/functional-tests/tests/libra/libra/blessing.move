//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
// Make sure that Coin1 and Coin2 are registered. Make sure that the rules
// relating to SCS and synthetic currencies are consistent
fun main() {
    assert(Libra::is_currency<Coin1>(), 1);
    assert(Libra::is_currency<Coin2>(), 2);
    assert(!Libra::is_synthetic_currency<Coin1>(), 2);
    assert(!Libra::is_synthetic_currency<Coin2>(), 3);
    assert(Libra::is_SCS_currency<Coin1>(), 4);
    assert(Libra::is_SCS_currency<Coin2>(), 5);
    Libra::assert_is_currency<Coin1>();
    Libra::assert_is_currency<Coin2>();
    Libra::assert_is_SCS_currency<Coin1>();
    Libra::assert_is_SCS_currency<Coin2>();
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::Libra;
use 0x1::LBR::LBR;
fun main() {
    Libra::assert_is_SCS_currency<LBR>();
}
}
// check: "Keep(ABORTED { code: 257,"

//! new-transaction
script {
use 0x1::Libra;
fun main() {
    Libra::assert_is_currency<u64>();
}
}
// check: "Keep(ABORTED { code: 261,"

//! new-transaction
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::FixedPoint32;
fun main(account: &signer) {
    Libra::update_lbr_exchange_rate<Coin1>(account, FixedPoint32::create_from_rational(1, 3));
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    Libra::update_minting_ability<Coin1>(account, false);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
module Holder {
    resource struct Holder<T> { x: T }
    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Holder<T>{ x })
    }

    public fun get<T>(addr: address): T
    acquires Holder {
       let Holder<T> { x } = move_from<Holder<T>>(addr);
       x
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::FixedPoint32;
use {{default}}::Holder;
fun main(lr_account: &signer) {
    let (a, b) = Libra::register_currency<Coin1>(
        lr_account,
        FixedPoint32::create_from_rational(1, 1),
        false,
        1000,
        10,
        b"ABC",
    );

    Holder::hold(lr_account, a);
    Holder::hold(lr_account, b);
}
}
// check: "Keep(ABORTED { code: 262,"
