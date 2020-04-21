// Add simple validator to LibraSystem's validator set.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: bob
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef");
    let config = ValidatorConfig2::get_config({{bob}});
    let consensus_pk = ValidatorConfig2::get_consensus_pubkey(&config);
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
use 0x0::LibraSystem2;
fun main() {
    let validator_size = LibraSystem2::validator_set_size();
    0x0::Transaction::assert(validator_size == 0, 99);
    LibraSystem2::add_validator({{bob}});
    validator_size = LibraSystem2::validator_set_size();
    0x0::Transaction::assert(validator_size == 1, 99);
}

// check: EXECUTED
