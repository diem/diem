//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
use 0x1::LibraTimestamp;
fun main(account: &signer) {
    LibraTimestamp::reset_time_has_started_for_test();
    Libra::initialize(account);
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE

//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
fun main(account: &signer) {
    Libra::initialize(account);
}
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::LibraTimestamp;
fun main(account: &signer) {
    LibraTimestamp::reset_time_has_started_for_test();
    Libra::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
