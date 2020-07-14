//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::initialize(account);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 0

//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::set(account, 0);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 1
