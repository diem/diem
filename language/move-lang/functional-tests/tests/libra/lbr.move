//! account: alice, 1000Coin1
//! account: bob, 1000Coin2

//! new-transaction
script {
use 0x1::Libra;
use 0x1::LBR::LBR;
fun main() {
    assert(Libra::approx_lbr_for_value<LBR>(10) == 10, 1);
    assert(Libra::scaling_factor<LBR>() == 1000000, 2);
    assert(Libra::fractional_part<LBR>() == 1000, 3);
}
}
// check: EXECUTED

// minting LBR via Libra::mint should not work
//! new-transaction
//! sender: blessed
script {
use 0x1::LBR::LBR;
use 0x1::Libra;
fun main(account: &signer) {
    let lbr = Libra::mint<LBR>(account, 1000);
    Libra::destroy_zero(lbr)
}
}
// check: MISSING_DATA

// burning LBR via Libra::burn should not work. This is cumbersome to test directly, so instead test
// that the Association account does not have a BurnCapability<LBR>
//! new-transaction
//! sender: blessed
script {
use 0x1::LBR::LBR;
use 0x1::Libra;
use 0x1::Offer;
fun main(account: &signer) {
   Offer::create(account, Libra::remove_burn_capability<LBR>(account), {{alice}});
}
}
// check: "ABORTED { code: 4"

// Check that is_lbr only returns true for LBR
//! new-transaction
script {
use 0x1::LBR;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
fun main() {
    assert(LBR::is_lbr<LBR::LBR>(), 1);
    assert(!LBR::is_lbr<Coin1>(), 2);
    assert(!LBR::is_lbr<Coin2>(), 3);
    assert(!LBR::is_lbr<bool>(), 4);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin2>(&with_cap, {{alice}}, 100, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! args: 10
stdlib_script::mint_lbr
// check: "Keep(ABORTED { code: 4615,"

//! new-transaction
//! sender: alice
//! args: 0
stdlib_script::mint_lbr
// check: "Keep(ABORTED { code: 775,"

//! new-transaction
module Holder {
    resource struct Holder<T> { x: T }
    public fun hold<T>(account: &signer, x: T)  {
        move_to(account, Holder<T> { x })
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin2::Coin2;
use 0x1::Coin1::Coin1;
use 0x1::LBR;
use {{default}}::Holder;
fun main(account: &signer) {
    let (c1_amount, c2_amount) = LBR::calculate_component_amounts_for_lbr(10);
    let c1 = Libra::mint<Coin1>(account, c1_amount + 1);
    let c2 = Libra::mint<Coin2>(account, c2_amount);
    Holder::hold(account, LBR::create(10, c1, c2));
}
}
// check: "Keep(ABORTED { code: 263,"

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin2::Coin2;
use 0x1::Coin1::Coin1;
use 0x1::LBR;
use {{default}}::Holder;
fun main(account: &signer) {
    let (c1_amount, c2_amount) = LBR::calculate_component_amounts_for_lbr(10);
    let c1 = Libra::mint<Coin1>(account, c1_amount);
    let c2 = Libra::mint<Coin2>(account, c2_amount + 1);
    Holder::hold(account, LBR::create(10, c1, c2));
}
}
// check: "Keep(ABORTED { code: 519,"
