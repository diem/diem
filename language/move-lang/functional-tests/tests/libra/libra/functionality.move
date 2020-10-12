//! account: bob, 0Coin1

module BurnCapabilityHolder {
    use 0x1::Libra;
    resource struct Holder<Token> {
        cap: Libra::BurnCapability<Token>,
    }

    public fun hold<Token>(account: &signer, cap: Libra::BurnCapability<Token>) {
        move_to(account, Holder<Token>{ cap })
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::Offer;
fun main(account: &signer) {
    let coin1_tmp = Libra::mint<Coin1>(account, 10000);
    assert(Libra::value<Coin1>(&coin1_tmp) == 10000, 0);

    let (coin1_tmp1, coin1_tmp2) = Libra::split(coin1_tmp, 5000);
    assert(Libra::value<Coin1>(&coin1_tmp1) == 5000 , 0);
    assert(Libra::value<Coin1>(&coin1_tmp2) == 5000 , 2);
    let tmp = Libra::withdraw(&mut coin1_tmp1, 1000);
    assert(Libra::value<Coin1>(&coin1_tmp1) == 4000 , 4);
    assert(Libra::value<Coin1>(&tmp) == 1000 , 5);
    Libra::deposit(&mut coin1_tmp1, tmp);
    assert(Libra::value<Coin1>(&coin1_tmp1) == 5000 , 6);
    let coin1_tmp = Libra::join(coin1_tmp1, coin1_tmp2);
    assert(Libra::value<Coin1>(&coin1_tmp) == 10000, 7);
    Offer::create(account, coin1_tmp, {{blessed}});

    Libra::destroy_zero(Libra::zero<Coin1>());
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    Libra::destroy_zero(Libra::mint<Coin1>(account, 1));
}
}
// check: "Keep(ABORTED { code: 2055,"

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
    use 0x1::Libra;
    use 0x1::Coin1::Coin1;
    fun main()  {
        let coins = Libra::zero<Coin1>();
        Libra::approx_lbr_for_coin<Coin1>(&coins);
        Libra::destroy_zero(coins);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
    use 0x1::Libra;
    fun main()  {
        Libra::destroy_zero(
            Libra::zero<u64>()
        );
    }
}
// check: "Keep(ABORTED { code: 261"

//! new-transaction
script {
    use 0x1::Libra;
    use 0x1::LBR::LBR;
    use 0x1::Coin1::Coin1;
    fun main()  {
        assert(!Libra::is_synthetic_currency<Coin1>(), 9);
        assert(Libra::is_synthetic_currency<LBR>(), 10);
        assert(!Libra::is_synthetic_currency<u64>(), 11);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use 0x1::Libra;
    use 0x1::Coin1::Coin1;
    use {{default}}::BurnCapabilityHolder;
    fun main(account: &signer)  {
        BurnCapabilityHolder::hold(
            account,
            Libra::remove_burn_capability<Coin1>(account)
        );
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
module Holder {
    resource struct Holder<T> { x: T }
    public fun hold<T>(account: &signer, x: T)  {
        move_to(account, Holder<T> { x })
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::FixedPoint32;
use {{default}}::Holder;
fun main(account: &signer) {
    let (mint_cap, burn_cap) = Libra::register_currency<u64>(
        account, FixedPoint32::create_from_rational(1, 1), true, 10, 10, b"wat"
    );
    Libra::publish_burn_capability(account, burn_cap);
    Holder::hold(account, mint_cap);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::FixedPoint32;
use {{default}}::Holder;
fun main(account: &signer) {
    let (mint_cap, burn_cap) = Libra::register_currency<u64>(
        account, FixedPoint32::create_from_rational(1, 1), true, 10, 10, b"wat"
    );
    Holder::hold(account, mint_cap);
    Holder::hold(account, burn_cap);
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::FixedPoint32;
fun main(account: &signer) {
    Libra::register_SCS_currency<u64>(
        account, account, FixedPoint32::create_from_rational(1, 1), 10, 10, b"wat"
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use {{default}}::Holder;
fun main(account: &signer) {
    Holder::hold(account, Libra::create_preburn<Coin1>(account));
}
}
// check: "Keep(ABORTED { code: 258,")

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::LBR::LBR;
fun main(account: &signer) {
    Libra::publish_preburn_to_account<LBR>(account, account);
}
}
// check: "Keep(ABORTED { code: 1539,")

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    Libra::publish_preburn_to_account<Coin1>(account, account);
}
}
// check: "Keep(ABORTED { code: 1539,")

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let coin1_tmp = Libra::mint<Coin1>(account, 1);
    let tmp = Libra::withdraw(&mut coin1_tmp, 10);
    Libra::destroy_zero(tmp);
    Libra::destroy_zero(coin1_tmp);
}
}
// check: "Keep(ABORTED { code: 2824,"

//! new-transaction
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::LBR::LBR;
fun main() {
    assert(Libra::is_SCS_currency<Coin1>(), 99);
    assert(!Libra::is_SCS_currency<LBR>(), 98);
    assert(!Libra::is_synthetic_currency<Coin1>(), 97);
    assert(Libra::is_synthetic_currency<LBR>(), 96);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::CoreAddresses;
fun main(account: &signer) {
    CoreAddresses::assert_currency_info(account)
}
}
// check: "Keep(ABORTED { code: 1026,"

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
fun main(tc_account: &signer) {
    let max_u64 = 18446744073709551615;
    let coin1 = Libra::mint<Coin1>(tc_account, max_u64);
    let coin2 = Libra::mint<Coin1>(tc_account, 1);
    Libra::deposit(&mut coin1, coin2);
    Libra::destroy_zero(coin1);
}
}
// check: "Keep(ABORTED { code: 1800,"
