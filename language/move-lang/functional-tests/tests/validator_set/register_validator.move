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
//! sender: libraroot
script{
    use 0x1::LibraAccount;

    // register Alice as a validator candidate
    fun main(creator: &signer) {
//        LibraAccount::create_validator_account(
//            creator, &r, 0xAA, x"00000000000000000000000000000000"
        LibraAccount::create_validator_account(
            creator, 0xAA, x"00000000000000000000000000000000", b"owner_name"
        );
    }
}

// check: EXECUTED

// TODO(valerini): enable the following test once the sender format is supported
// //! new-transaction
// //! sender: 0xAA
// script{

//     // register Alice as a validator candidate, then rotate a key + check that it worked.
//     fun main(account: &signer) {
//         // Alice registers as a validator candidate

//         // Rotating the consensus_pubkey should work

//         // Rotating the validator's full config
//     }
// }

// // check: EXECUTED
