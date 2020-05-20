// Register alice as bob's delegate,
// test all possible key rotations:
// bob's key by bob - aborts
// bob's key by alice - executes
// alice's key by bob - aborts
// alice's key by alice - executes

//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! sender: bob
script {
use 0x0::ValidatorConfig;
fun main() {
    // set alice to change bob's key
    ValidatorConfig::set_operator({{alice}});
}
}

// check: EXECUTED

//! new-transaction
//! sender: bob
// check bob can not rotate his consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_consensus_pubkey({{bob}}, x"30");
}
}

// check: ABORTED

//! new-transaction
//! sender: bob
// check bob can not rotate alice's consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_consensus_pubkey({{alice}}, x"30");
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
// check alice can rotate bob's consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_consensus_pubkey({{bob}}, x"30");
    0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})) == x"30", 99);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
// check alice can rotate her consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_consensus_pubkey({{alice}}, x"20");
    0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"20", 99);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
// check alice can rotate her consensus key
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::set_consensus_pubkey({{alice}}, x"30");
    0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"30", 99);
}
}

// check: EXECUTED
