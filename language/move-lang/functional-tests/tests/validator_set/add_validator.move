// Add simple validator to LibraSystem's validator set.

//! account: bob, 1000000, 0, validator

//! sender: bob
script {
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    fun main() {
        // test bob is a validator
        assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
        assert(LibraSystem::is_validator({{bob}}) == true, 98);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
fun main(creator: &signer) {
//    LibraAccount::create_validator_account(
//        creator, &r, 0xAA, x"00000000000000000000000000000000"
    LibraAccount::create_validator_account(
        creator, 0xAA, x"00000000000000000000000000000000", b"owner_name"
    );

}
}

// check: EXECUTED

// TODO(valerini): enable the following test once the sender format is supported
// //! new-transaction
// //! sender: 0xAA
// script {
// fun main() {
//
//     // add itself as a validator
// }
// }
//
// // check: EXECUTED
// // check: NewEpochEvent
