//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! new-transaction
//! sender: alice
script{
use 0x0::ValidatorConfig;

// register Alice as a validator candidate
fun main() {
    ValidatorConfig::register_candidate_validator(x"10", x"20", x"30", x"40", x"50", x"60");
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
// We don't allow two reconfiguration events to happen within the same block. That includes following examples:
// 1. Add then remove.
// 2. Double rotation.
// etc.

script{
use 0x0::LibraSystem;
fun main() {
    LibraSystem::add_validator({{alice}});
    LibraSystem::remove_validator({{vivian}});
}
}

// check: ABORT
// check: 23

//! new-transaction
//! sender: vivian
// validators: vivian, viola
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"40");
    LibraSystem::update_and_reconfigure();

    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"50");
    LibraSystem::update_and_reconfigure();
}
}

// check: ABORT
// check: 23
