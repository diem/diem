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
