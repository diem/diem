//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::set(account, 0);
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::set(account, 0);
}
}
// check: "Keep(ABORTED { code: 7,"
