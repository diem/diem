// Creating an account of any type at the reserved address 0x0 should fail


//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
fun main(account: &signer) {
//    LibraAccount::create_unhosted_account<LBR>(
    LibraAccount::create_unhosted_account<LBR>(
        account, 0x0, x"00000000000000000000000000000000", false);
}
}
// check: ABORTED
// check: 0
