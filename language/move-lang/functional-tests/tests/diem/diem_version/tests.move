//! new-transaction
script{
use DiemFramework::DiemVersion;
fun main(account: signer) {
    let account = &account;
    DiemVersion::initialize(account, 1);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
script{
use DiemFramework::DiemVersion;
fun main(account: signer) {
    let account = &account;
    DiemVersion::set(account, 0);
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: diemroot
script{
use DiemFramework::DiemVersion;
fun main(account: signer) {
    let account = &account;
    DiemVersion::set(account, 0);
}
}
// check: "Keep(ABORTED { code: 7,"
