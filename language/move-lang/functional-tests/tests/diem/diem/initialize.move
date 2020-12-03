//! new-transaction
//! sender: diemroot
script {
use 0x1::Diem;
fun main(account: &signer) {
    Diem::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
