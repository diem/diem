//! account: bob, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: alice

//! new-transaction
script {
use 0x0::LibraSystem;
fun main(account: &signer) {
    LibraSystem::initialize_validator_set(account);
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
//! sender: bob
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_operator(0x0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
use 0x0::Transaction;
fun main() {
    ValidatorConfig::set_operator(Transaction::sender());
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
// delegate to alice
fun main() {
    ValidatorConfig::set_operator({{alice}});
    ValidatorConfig::remove_operator();
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_consensus_pubkey({{vivian}}, x"");
}
}
// check: ABORTED
// check: 1101

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
fun main(account: &signer) {
    ValidatorConfig::set_config(account, {{vivian}}, x"", x"", x"", x"", x"");
}
}
// check: ABORTED
// check: 1101
