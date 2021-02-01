//! new-transaction
//! sender: diemroot
script {
use 0x1::Diem;
use 0x1::XUS::XUS;
// Make sure that XUS is registered. Make sure that the rules
// relating to SCS and synthetic currencies are consistent
fun main() {
    assert(Diem::is_currency<XUS>(), 1);
    assert(!Diem::is_synthetic_currency<XUS>(), 2);
    assert(Diem::is_SCS_currency<XUS>(), 4);
    Diem::assert_is_currency<XUS>();
    Diem::assert_is_SCS_currency<XUS>();
}
}

//! new-transaction
script {
use 0x1::Diem;
use 0x1::XDX::XDX;
fun main() {
    Diem::assert_is_SCS_currency<XDX>();
}
}

//! new-transaction
script {
use 0x1::Diem;
fun main() {
    Diem::assert_is_currency<u64>();
}
}

//! new-transaction
script {
use 0x1::Diem;
use 0x1::XUS::XUS;
use 0x1::FixedPoint32;
fun main(account: &signer) {
    Diem::update_xdx_exchange_rate<XUS>(account, FixedPoint32::create_from_rational(1, 3));
}
}

//! new-transaction
script {
use 0x1::Diem;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    Diem::update_minting_ability<XUS>(account, false);
}
}

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

//! new-transaction
//! sender: diemroot
script {
use 0x1::Diem;
use 0x1::XUS::XUS;
use 0x1::FixedPoint32;
use {{default}}::Holder;
fun main(dr_account: &signer) {
    let (a, b) = Diem::register_currency<XUS>(
        dr_account,
        FixedPoint32::create_from_rational(1, 1),
        false,
        1000,
        10,
        b"ABC",
    );

    Holder::hold(dr_account, a);
    Holder::hold(dr_account, b);
}
}

//! new-transaction
//! sender: diemroot
script {
use 0x1::Diem;
use 0x1::FixedPoint32;
use {{default}}::Holder;
fun main(dr_account: &signer) {
    let (a, b) = Diem::register_currency<u64>(
        dr_account,
        FixedPoint32::create_from_rational(1,1),
        false,
        0, // scaling factor
        100,
        x""
    );
    Holder::hold(dr_account, a);
    Holder::hold(dr_account, b);
}
}

//! new-transaction
//! sender: diemroot
script {
use 0x1::Diem;
use 0x1::FixedPoint32;
use {{default}}::Holder;
fun main(dr_account: &signer) {
    let (a, b) = Diem::register_currency<u64>(
        dr_account,
        FixedPoint32::create_from_rational(1,1),
        false,
        1000000000000000, // scaling factor > MAX_SCALING_FACTOR
        100,
        x""
    );
    Holder::hold(dr_account, a);
    Holder::hold(dr_account, b);
}
}
