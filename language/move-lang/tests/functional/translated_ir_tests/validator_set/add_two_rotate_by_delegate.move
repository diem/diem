// Add bob and alice as validators
// Register alice as bob's delegate,
// test all possible key rotations:
// bob's key by bob - aborts
// bob's key by alice - executes
// alice's key by bob - aborts
// alice's key by alice - executes

//! account: alice
//! account: bob
//! account: carrol, 1000000, 0, validator

//! sender: bob
script {
use 0x0::ValidatorConfig;
// initialize bob as validator
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
    // set alice to change bob's key
    ValidatorConfig::set_delegated_account({{alice}});
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x0::ValidatorConfig;
// initialize alice as validator
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}
}

// check: EXECUTED

//! block-prologue
//! proposer: carrol
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraSystem;
fun main() {
    // add validator
    LibraSystem::add_validator({{bob}});
}
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: carrol
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraSystem;
fun main() {
    // add validator
    LibraSystem::add_validator({{alice}});
}
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: carrol
//! block-time: 4

// check: EXECUTED

//! new-transaction
//! sender: bob
// check bob can not rotate his consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"30");
}
}

// check: ABORTED

//! new-transaction
//! sender: bob
// check bob can not rotate his consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::rotate_consensus_pubkey({{bob}}, x"30");
}
}

// check: ABORTED

//! new-transaction
//! sender: bob
// check bob can not rotate alice's consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::rotate_consensus_pubkey({{alice}}, x"30");
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
// check alice can rotate bob's consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})) == x"beefbeef", 99);
    ValidatorConfig::rotate_consensus_pubkey({{bob}}, x"30");
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})) == x"30", 99);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
// check alice can rotate her consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"beefbeef", 99);
    ValidatorConfig::rotate_consensus_pubkey({{alice}}, x"20");
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"20", 99);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
// check alice can rotate her consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"30");
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"30", 99);
}
}

// check: EXECUTED
