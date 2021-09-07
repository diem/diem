// Add simple validator to DiemSystem's validator set.

//! account: bob, 1000000, 0, validator
//! account: alex, 0, 0, address

//! sender: bob
script {
    use DiemFramework::DiemSystem;
    use DiemFramework::ValidatorConfig;
    fun main() {
        // test bob is a validator
        assert(ValidatorConfig::is_valid(@{{bob}}) == true, 98);
        assert(DiemSystem::is_validator(@{{bob}}) == true, 98);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
use DiemFramework::DiemAccount;
fun main(creator: signer) {
    let creator = &creator;
//    DiemAccount::create_validator_account(
//        creator, &r, 0xAA, x"00000000000000000000000000000000"
    DiemAccount::create_validator_account(
        creator, @0xAA, x"00000000000000000000000000000000", b"owner_name"
    );

}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
//! args: 0, {{alex}}, {{alex::auth_key}}, b"alex"
stdlib_script::AccountCreationScripts::create_validator_account
// check: "Keep(EXECUTED)"

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
// // check: "Keep(EXECUTED)"
// // check: NewEpochEvent
