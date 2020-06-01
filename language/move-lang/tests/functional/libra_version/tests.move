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
    LibraVersion::set(account, 0);
}
}
// check: ABORTED
// check: 25
