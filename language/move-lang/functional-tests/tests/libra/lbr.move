//! account: alice

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
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
use 0x1::Libra;
fun main(account: &signer) {
   LibraAccount::deposit_to(account, Libra::mint<LBR>(account, 1000));
}
}
// check: MISSING_DATA

// burning LBR via Libra::burn should not work. This is cumbersome to test directly, so instead test
// that the Association account does not have a BurnCapability
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
// check: MISSING_DATA

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
