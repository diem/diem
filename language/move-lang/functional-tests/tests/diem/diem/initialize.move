//! new-transaction
//! sender: diemroot
script {
use 0x1::Diem;
fun main(account: signer) {
    let account = &account;
    Diem::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
