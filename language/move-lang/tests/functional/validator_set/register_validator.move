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
//! sender: association
script{
    use 0x0::LBR;
    use 0x0::LibraAccount;

    // register Alice as a validator candidate
    fun main() {
        LibraAccount::create_validator_account<LBR::T>(0xAA, x"00000000000000000000000000000000");
    }
}

// check: EXECUTED

// TODO(valerini): enable the following test once the sender format is supported
// //! new-transaction
// //! sender: 0xAA
// script{
//     use 0x0::ValidatorConfig;

//     // register Alice as a validator candidate, then rotate a key + check that it worked.
//     fun main() {
//         // Alice registers as a validator candidate
//         0x0::Transaction::assert(!ValidatorConfig::is_valid(0x0::Transaction::sender()), 9);
//         ValidatorConfig::set_config(0xAA, x"10", x"20", x"30", x"40", x"50", x"60");

//         // Rotating the consensus_pubkey should work
//         let config = ValidatorConfig::get_config(0x0::Transaction::sender());
//         0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"10", 11);
//         ValidatorConfig::set_consensus_pubkey(0xAA, x"70");
//         0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"10", 12);
//         config = ValidatorConfig::get_config(0x0::Transaction::sender());
//         0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"70", 15);

//         // Rotating the validator's full config
//         ValidatorConfig::set_config(0xAA, x"70", x"80", x"90", x"100", x"110", x"120");
//         config = ValidatorConfig::get_config(0x0::Transaction::sender());
//         0x0::Transaction::assert(*ValidatorConfig::get_validator_network_identity_pubkey(&config) == x"90", 13);
//         ValidatorConfig::set_config(0xAA, x"70", x"80", x"55", x"100", x"110", x"120");
//         config = ValidatorConfig::get_config(0x0::Transaction::sender());
//         0x0::Transaction::assert(*ValidatorConfig::get_validator_network_identity_pubkey(&config) == x"55", 14);
//     }
// }

// // check: EXECUTED
