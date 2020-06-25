// Creating an account of any type at the reserved address 0x0 should fail


//! new-transaction
//! sender: association
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
use 0x1::Roles::{Self, LibraRootRole};
fun main(account: &signer) {
    let cap = Roles::extract_privilege_to_capability<LibraRootRole>(account);
    LibraAccount::create_unhosted_account<LBR>(
        account, &cap, 0x0, x"00000000000000000000000000000000", false);
    Roles::restore_capability_to_privilege(account, cap);
}
}
// check: ABORTED
// check: 0
