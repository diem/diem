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

//! new-transaction
script {
use 0x0::Libra;
use 0x0::Coin1;
fun main() {
    Libra::grant_mint_capability_for_sender<Coin1::T>();
}
}
// check: ABORTED
// check: 0
