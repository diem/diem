//! new-transaction
//! sender: association
script {
use 0x0::Association;
// init the association
fun main() {
    Association::initialize();
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE

//! new-transaction
//! sender: config
script {
use 0x0::Libra;
fun main() {
    Libra::initialize();
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE
