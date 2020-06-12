//! account: bob, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: alice

//! new-transaction
script {
use 0x1::LibraSystem;
use 0x1::LibraConfig::CreateOnChainConfig;
use 0x1::Roles;
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
    LibraSystem::initialize_validator_set(account, &r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: ABORTED
// check: 1

//! new-transaction
script {
use 0x1::LibraSystem;
fun main(account: &signer) {
    LibraSystem::update_and_reconfigure(account);
}
}
// check: ABORTED
// check: 22

//! new-transaction
//! sender: bob
script {
use 0x1::ValidatorConfig;
fun main(account: &signer) {
    ValidatorConfig::set_operator(account, 0x0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::Signer;
use 0x1::ValidatorConfig;
fun main(account: &signer) {
    ValidatorConfig::set_operator(account, Signer::address_of(account))
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::ValidatorConfig;
// delegate to alice
fun main(account: &signer) {
    ValidatorConfig::set_operator(account, {{alice}});
    ValidatorConfig::remove_operator(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::ValidatorConfig;
fun main(account: &signer) {
    ValidatorConfig::set_consensus_pubkey(account, {{vivian}}, x"");
}
}
// check: ABORTED
// check: 1101

//! new-transaction
//! sender: bob
script {
use 0x1::ValidatorConfig;
fun main(account: &signer) {
    ValidatorConfig::set_config(account, {{vivian}}, x"", x"", x"", x"", x"");
}
}
// check: ABORTED
// check: 1101
