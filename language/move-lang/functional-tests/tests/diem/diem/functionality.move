//! account: bob, 0XUS

module {{default}}::Holder {
    struct Holder<T> has key { x: T }
    public fun hold<T: store>(account: &signer, x: T)  {
        move_to(account, Holder<T> { x })
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
use {{default}}::Holder;
fun main(account: signer) {
    let account = &account;
    let xus = Diem::mint<XUS>(account, 10000);
    assert(Diem::value<XUS>(&xus) == 10000, 0);

    let (xus1, xus2) = Diem::split(xus, 5000);
    assert(Diem::value<XUS>(&xus1) == 5000 , 0);
    assert(Diem::value<XUS>(&xus2) == 5000 , 2);
    let tmp = Diem::withdraw(&mut xus1, 1000);
    assert(Diem::value<XUS>(&xus1) == 4000 , 4);
    assert(Diem::value<XUS>(&tmp) == 1000 , 5);
    Diem::deposit(&mut xus1, tmp);
    assert(Diem::value<XUS>(&xus1) == 5000 , 6);
    let xus = Diem::join(xus1, xus2);
    assert(Diem::value<XUS>(&xus) == 10000, 7);
    Holder::hold(account, xus);

    Diem::destroy_zero(Diem::zero<XUS>());
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    Diem::destroy_zero(Diem::mint<XUS>(account, 1));
}
}
// check: "Keep(ABORTED { code: 2055,"

//! new-transaction
//! sender: bob
//! gas-currency: XUS
script {
    use DiemFramework::Diem;
    use DiemFramework::XUS::XUS;
    fun main()  {
        let coins = Diem::zero<XUS>();
        Diem::approx_xdx_for_coin<XUS>(&coins);
        Diem::destroy_zero(coins);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
    use DiemFramework::Diem;
    fun main()  {
        Diem::destroy_zero(
            Diem::zero<u64>()
        );
    }
}
// check: "Keep(ABORTED { code: 261"

//! new-transaction
script {
    use DiemFramework::Diem;
    use DiemFramework::XDX::XDX;
    use DiemFramework::XUS::XUS;
    fun main()  {
        assert(!Diem::is_synthetic_currency<XUS>(), 9);
        assert(Diem::is_synthetic_currency<XDX>(), 10);
        assert(!Diem::is_synthetic_currency<u64>(), 11);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use DiemFramework::Diem;
    use DiemFramework::XUS::XUS;
    use {{default}}::Holder;
    fun main(account: signer)  {
    let account = &account;
        Holder::hold(
            account,
            Diem::remove_burn_capability<XUS>(account)
        );
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::Diem;
use Std::FixedPoint32;
use {{default}}::Holder;
fun main(account: signer) {
    let account = &account;
    let (mint_cap, burn_cap) = Diem::register_currency<u64>(
        account, FixedPoint32::create_from_rational(1, 1), true, 10, 10, b"wat"
    );
    Diem::publish_burn_capability(account, burn_cap);
    Holder::hold(account, mint_cap);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::Diem;
use Std::FixedPoint32;
use {{default}}::Holder;
fun main(account: signer) {
    let account = &account;
    let (mint_cap, burn_cap) = Diem::register_currency<u64>(
        account, FixedPoint32::create_from_rational(1, 1), true, 10, 10, b"wat"
    );
    Holder::hold(account, mint_cap);
    Holder::hold(account, burn_cap);
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::Diem;
use Std::FixedPoint32;
fun main(account: signer) {
    let account = &account;
    Diem::register_SCS_currency<u64>(
        account, account, FixedPoint32::create_from_rational(1, 1), 10, 10, b"wat"
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
use {{default}}::Holder;
fun main(account: signer) {
    let account = &account;
    Holder::hold(account, Diem::create_preburn<XUS>(account));
}
}
// check: "Keep(ABORTED { code: 258,")

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::Diem;
use DiemFramework::XDX::XDX;
fun main(account: signer) {
    let account = &account;
    Diem::publish_preburn_queue_to_account_for_test<XDX>(account, account);
}
}
// check: "Keep(ABORTED { code: 1539,")

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    Diem::publish_preburn_queue_to_account_for_test<XUS>(account, account);
}
}
// check: "Keep(ABORTED { code: 1539,")

//! new-transaction
//! sender: blessed
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    let xus = Diem::mint<XUS>(account, 1);
    let tmp = Diem::withdraw(&mut xus, 10);
    Diem::destroy_zero(tmp);
    Diem::destroy_zero(xus);
}
}
// check: "Keep(ABORTED { code: 2568,"

//! new-transaction
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
use DiemFramework::XDX::XDX;
fun main() {
    assert(Diem::is_SCS_currency<XUS>(), 99);
    assert(!Diem::is_SCS_currency<XDX>(), 98);
    assert(!Diem::is_synthetic_currency<XUS>(), 97);
    assert(Diem::is_synthetic_currency<XDX>(), 96);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::CoreAddresses;
fun main(account: signer) {
    let account = &account;
    CoreAddresses::assert_currency_info(account)
}
}
// check: "Keep(ABORTED { code: 770,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::Diem;
use DiemFramework::XUS::XUS;
fun main(tc_account: signer) {
    let tc_account = &tc_account;
    let max_u64 = 18446744073709551615;
    let coin1 = Diem::mint<XUS>(tc_account, max_u64);
    let coin2 = Diem::mint<XUS>(tc_account, 1);
    Diem::deposit(&mut coin1, coin2);
    Diem::destroy_zero(coin1);
}
}
// check: "Keep(ABORTED { code: 1800,"
