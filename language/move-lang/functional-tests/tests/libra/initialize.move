//! new-transaction
//! sender: association
script {
use 0x0::Association;
// init the association
fun main(account: &signer) {
    Association::initialize(account);
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE

//! new-transaction
//! sender: config
script {
use 0x0::Libra;
fun main(account: &signer) {
    Libra::initialize(account);
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE
