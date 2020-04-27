// Add simple validator to LibraSystem's validator set.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: bob
use 0x0::ValidatorConfig;
fun main() {
    // test bob is a validator
    0x0::Transaction::assert(ValidatorConfig::has({{bob}}) == true, 98);
}

// check: EXECUTED

//! new-transaction
//! sender: alice
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
    let config = ValidatorConfig::get_config({{alice}});
    let consensus_pk = ValidatorConfig::get_consensus_pubkey(&config);
    let expected_pk = x"beefbeef";
    0x0::Transaction::assert(&consensus_pk == &expected_pk, 98);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem;
fun main() {
    let validator_size = LibraSystem::validator_set_size();
    0x0::Transaction::assert(validator_size == 1, 99);
    LibraSystem::add_validator({{alice}});
    validator_size = LibraSystem::validator_set_size();
    0x0::Transaction::assert(validator_size == 2, 99);
}

// check: EXECUTED
