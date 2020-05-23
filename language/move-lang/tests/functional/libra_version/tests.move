//! account: validator, 10000, 0, validator

//! new-transaction
script{
use 0x0::LibraVersion;
fun main() {
    LibraVersion::initialize();
}
}
// check: ABORTED
// check: 1

//! new-transaction
script{
use 0x0::LibraVersion;
fun main() {
    LibraVersion::set(0);
}
}
// check: ABORTED
// check: 25

//! new-transaction
script{
use 0x0::LibraVersion;
fun main() {
    LibraVersion::set(1);
}
}
// check: ABORTED
// check: 25

//! new-transaction
//! sender: config
script{
use 0x0::LibraVersion;
fun main() {
    LibraVersion::set(2);
}
}
// check: ABORTED
// check: 23

//! new-transaction
//! sender: config
script{
use 0x0::LibraVersion;
fun main() {
    LibraVersion::set(2);
}
}
// check: ABORTED
// check: 23

//! block-prologue
//! proposer: validator
//! block-time: 3

//! new-transaction
//! sender: config
script{
use 0x0::LibraVersion;
fun main() {
    LibraVersion::set(2);
}
}
// check: EXECUTED
