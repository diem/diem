//! new-transaction
//! sender: diemroot
script {
use DiemFramework::Diem;
fun main(account: signer) {
    let account = &account;
    Diem::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
