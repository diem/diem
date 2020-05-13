//! account: bob, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: alice

//! new-transaction
script {
use 0x0::LibraSystem;
fun main() {
    LibraSystem::initialize_validator_set();
}
}
// check: ABORTED
// check: 1

//! new-transaction
script {
use 0x0::LibraSystem;
fun main() {
    LibraSystem::update_and_reconfigure();
}
}
// check: ABORTED
// check: 22

//! new-transaction
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_delegated_account(0x0);
}
}
// check: ABORTED
// check: 5

//! new-transaction
script {
use 0x0::ValidatorConfig;
use 0x0::Transaction;
fun main() {
    ValidatorConfig::set_delegated_account(Transaction::sender());
}
}
// check: ABORTED
// check: 6

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
// delegate to alice
fun main() {
    ValidatorConfig::set_delegated_account({{alice}});
    ValidatorConfig::remove_delegated_account();
}
}

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
// delegate to alice
fun main() {
    ValidatorConfig::rotate_validator_network_identity_pubkey({{vivian}}, x"");
}
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
// delegate to alice
fun main() {
    ValidatorConfig::rotate_validator_network_address({{vivian}}, x"");
}
}
// check: ABORTED
// check: 1
