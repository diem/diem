//! account: alice
//! account: vivian, 100000, 0, validator
//! account: viola, 100000, 0, validator

// check that the validator account config works
script{
    use 0x1::LibraSystem;
    use 0x1::Signer;

    fun main(account: &signer) {
        let sender = Signer::address_of(account);
        assert(!LibraSystem::is_validator(sender), 1);
        assert(!LibraSystem::is_validator({{alice}}), 2);
        assert(LibraSystem::is_validator({{vivian}}), 3);
        assert(LibraSystem::is_validator({{viola}}), 4);
        // number of validators should equal the number we declared
        assert(LibraSystem::validator_set_size() == 2, 5);
        assert(LibraSystem::get_ith_validator_address(1) == {{vivian}}, 6);
        assert(LibraSystem::get_ith_validator_address(0) == {{viola}}, 7);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
    use 0x1::LibraSystem;
    use 0x1::Signer;

    // check that sending from validator accounts works
    fun main(account: &signer) {
        let sender = Signer::address_of(account);
        assert(LibraSystem::is_validator(sender), 8);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraAccount;
    use 0x1::Roles::{Self, LibraRootRole};

    // register Alice as a validator candidate
    fun main(creator: &signer) {
        let r = Roles::extract_privilege_to_capability<LibraRootRole>(creator);
        LibraAccount::create_validator_account(
            creator, &r, 0xAA, x"00000000000000000000000000000000"
        );
        Roles::restore_capability_to_privilege(creator, r);
    }
}

// check: EXECUTED

// TODO(valerini): enable the following test once the sender format is supported
// //! new-transaction
// //! sender: 0xAA
// script{
//     use 0x1::Signer;
//     use 0x1::ValidatorConfig;

//     // register Alice as a validator candidate, then rotate a key + check that it worked.
//     fun main(account: &signer) {
//         let sender = Signer::address_of(account);
//         // Alice registers as a validator candidate
//         assert(!ValidatorConfig::is_valid(sender), 9);
//         ValidatorConfig::set_config(0xAA, x"10", x"20", x"30", x"40", x"50", x"60");

//         // Rotating the consensus_pubkey should work
//         let config = ValidatorConfig::get_config(sender);
//         assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"10", 11);
//         ValidatorConfig::set_consensus_pubkey(0xAA, x"70");
//         assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"10", 12);
//         config = ValidatorConfig::get_config(sender);
//         assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"70", 15);

//         // Rotating the validator's full config
//         ValidatorConfig::set_config(0xAA, x"70", x"80", x"90", x"100", x"110", x"120");
//         config = ValidatorConfig::get_config(sender);
//         assert(*ValidatorConfig::get_validator_network_identity_pubkey(&config) == x"90", 13);
//         ValidatorConfig::set_config(0xAA, x"70", x"80", x"55", x"100", x"110", x"120");
//         config = ValidatorConfig::get_config(sender);
//         assert(*ValidatorConfig::get_validator_network_identity_pubkey(&config) == x"55", 14);
//     }
// }

// // check: EXECUTED
