//! new-transaction
script{
use 0x0::LibraVersion;
fun main(account: &signer) {
    LibraVersion::initialize(account);
}
}
// check: ABORTED
// check: 1

//! new-transaction
script{
use 0x0::LibraVersion;
fun main(account: &signer) {
    LibraVersion::set(0, account);
}
}
// check: ABORTED
// check: 25
