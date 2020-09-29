// Creating an account of any type at the reserved address 0x0 or core module address 0x1 should fail


//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::create_parent_vasp_account<Coin1>(
        account, 0x0, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 2567,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::create_parent_vasp_account<Coin1>(
        account, 0x0, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
fun main(account: &signer) {
    LibraAccount::create_parent_vasp_account<LBR>(
        account, 0x1, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 6151,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
fun main(account: &signer) {
    LibraAccount::create_parent_vasp_account<LBR>(
        account, 0x1, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 258,"
