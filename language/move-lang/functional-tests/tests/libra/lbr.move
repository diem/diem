//! account: alice

//! new-transaction
script {
use 0x0::Libra;
use 0x0::LBR::LBR;
fun main() {
    assert(Libra::approx_lbr_for_value<LBR>(10) == 10, 1);
    assert(Libra::scaling_factor<LBR>() == 1000000, 2);
    assert(Libra::fractional_part<LBR>() == 1000, 3);
}
}
// check: EXECUTED

// minting various amounts of LBR via LBR::mint should work
//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::LBR;
fun main(account: &signer) {
   LibraAccount::deposit(account, {{alice}}, LBR::mint(account, 1));
   LibraAccount::deposit(account, {{alice}}, LBR::mint(account,  2));
   LibraAccount::deposit(account, {{alice}}, LBR::mint(account, 77));
   LibraAccount::deposit(account, {{alice}}, LBR::mint(account, 100));
   LibraAccount::deposit(account, {{alice}}, LBR::mint(account, 1589));
}
}
// check: EXECUTED

// minting LBR via Libra::mint should not work
//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::LBR::LBR;
use 0x0::Libra;
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
use 0x0::LBR::LBR;
use 0x0::Libra;
use 0x0::Offer;
fun main(account: &signer) {
   Offer::create(account, Libra::remove_burn_capability<LBR>(account), {{alice}});
}
}
// check: MISSING_DATA
