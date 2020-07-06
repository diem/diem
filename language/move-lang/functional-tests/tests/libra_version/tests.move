//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::initialize(account);
}
}
// check: ABORTED
// check: 0

//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::set(account, 0);
}
}
// check: ABORTED
// check: 1
