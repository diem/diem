// Creating an account of any type at the reserved address 0x0 or core module address 0x1 should fail


//! new-transaction
//! sender: blessed
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    DiemAccount::create_parent_vasp_account<XUS>(
        account, @0x0, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 2567,"

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(account: signer) {
    let account = &account;
    DiemAccount::create_parent_vasp_account<XUS>(
        account, @0x0, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::DiemAccount;
use DiemFramework::XDX::XDX;
fun main(account: signer) {
    let account = &account;
    DiemAccount::create_parent_vasp_account<XDX>(
        account, @0x1, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 6151,"

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::DiemAccount;
use DiemFramework::XDX::XDX;
fun main(account: signer) {
    let account = &account;
    DiemAccount::create_parent_vasp_account<XDX>(
        account, @0x1, x"00000000000000000000000000000000", x"", false);
}
}
// check: "Keep(ABORTED { code: 258,"
