//! account: alice
//! account: vivian, 100000, 0, validator
//! account: viola, 100000, 0, validator

// check that the validator account config works
script{
use 0x0::LibraSystem;

fun main() {
    0x0::Transaction::assert(!LibraSystem::is_validator(0x0::Transaction::sender()), 1);
    0x0::Transaction::assert(!LibraSystem::is_validator({{alice}}), 2);
    0x0::Transaction::assert(LibraSystem::is_validator({{vivian}}), 3);
    0x0::Transaction::assert(LibraSystem::is_validator({{viola}}), 4);
    // number of validators should equal the number we declared
    0x0::Transaction::assert(LibraSystem::validator_set_size() == 2, 5);
    0x0::Transaction::assert(LibraSystem::get_ith_validator_address(1) == {{vivian}}, 6);
    0x0::Transaction::assert(LibraSystem::get_ith_validator_address(0) == {{viola}}, 7);
}
}

// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraSystem;

// check that sending from validator accounts works
fun main() {
    0x0::Transaction::assert(LibraSystem::is_validator(0x0::Transaction::sender()), 8);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script{
use 0x0::ValidatorConfig;

// register Alice as a validator candidate, then rotate a key + check that it worked.
fun main() {
    // Alice registers as a validator candidate
    0x0::Transaction::assert(!ValidatorConfig::has(0x0::Transaction::sender()), 9);
    ValidatorConfig::register_candidate_validator(x"10", x"20", x"30", x"40", x"50", x"60");
    0x0::Transaction::assert(ValidatorConfig::has(0x0::Transaction::sender()), 10);

    // Rotating the consensus_pubkey should work
    let config = ValidatorConfig::get_config(0x0::Transaction::sender());
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&config) == x"10", 11);
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"70");
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&config) == x"10", 12);
    config = ValidatorConfig::get_config(0x0::Transaction::sender());
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&config) == x"70", 15);

    // Rotating the validator_network_pubkey should work
    0x0::Transaction::assert(ValidatorConfig::get_validator_network_identity_pubkey(&config) == x"30", 13);
    ValidatorConfig::rotate_validator_network_identity_pubkey({{alice}}, x"80");
    config = ValidatorConfig::get_config(0x0::Transaction::sender());
    0x0::Transaction::assert(ValidatorConfig::get_validator_network_identity_pubkey(&config) == x"80", 14);
}
}

// check: EXECUTED
