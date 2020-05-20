//! account: alice

//! new-transaction
//! sender: association
script {
use 0x0::Libra;
use 0x0::LBR;
use 0x0::Transaction;
fun main() {
    Transaction::assert(Libra::approx_lbr_for_value<LBR::T>(10) == 10, 1);
    Transaction::assert(Libra::scaling_factor<LBR::T>() == 1000000, 2);
    Transaction::assert(Libra::fractional_part<LBR::T>() == 1000, 3);
}
}
// check: EXECUTED

// minting various amounts of LBR via LBR::mint should work
//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
use 0x0::LBR;
fun main() {
   LibraAccount::deposit({{alice}}, LBR::mint(1));
   LibraAccount::deposit({{alice}}, LBR::mint(2));
   LibraAccount::deposit({{alice}}, LBR::mint(77));
   LibraAccount::deposit({{alice}}, LBR::mint(100));
   LibraAccount::deposit({{alice}}, LBR::mint(1589));
}
}
// check: EXECUTED

// minting LBR via Libra::mint should not work
//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
fun main() {
   LibraAccount::deposit_to_sender(Libra::mint<LBR::T>(1000));
}
}
// check: MISSING_DATA

// burning LBR via Libra::burn should not work. This is cumbersome to test directly, so instead test
// that the Association account does not have a BurnCapability
//! new-transaction
//! sender: association
script {
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Offer;
fun main() {
   Offer::create(Libra::remove_burn_capability<LBR::T>(), {{alice}});
}
}
// check: MISSING_DATA
