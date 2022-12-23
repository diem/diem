//! account: bob, 0, 0, address
//! account: vasp, 0, 0, address
//! account: alice, 0, 0, address

// This is only here because cargo test complains about empty transactions!

//! sender: diemroot
//! execute-as: bob
script {
use Std::Signer;
fun main(dr: signer, bob: signer) {
    assert(Signer::address_of(&dr) == @DiemRoot, 0);
    assert(Signer::address_of(&bob) == @{{bob}}, 1);
}
}


// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::grant_diem_root_role(account);
// }
// }
// // check: "Keep(ABORTED { code: 1,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::grant_treasury_compliance_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 1,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::new_designated_dealer_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 258,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::new_designated_dealer_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 258,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::DiemAccount;
// use DiemFramework::XUS::XUS;
// fun main(account: signer) {
//     let account = &account;
//     DiemAccount::create_designated_dealer<XUS>(
//         account,
//         @{{bob}},
//         {{bob::auth_key}},
//         b"bob",
//         false
//     );
// }
// }
// // check: "Keep(ABORTED { code: 258,"

// //! new-transaction
// //! sender: blessed
// script {
// use DiemFramework::DiemAccount;
// fun main(account: signer) {
//     let account = &account;
//     DiemAccount::create_validator_account(
//         account,
//         @{{bob}},
//         {{bob::auth_key}},
//         b"bob",
//     );
// }
// }
// // check: "Keep(ABORTED { code: 2,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::new_validator_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 6,"

// //! new-transaction
// //! sender: blessed
// script {
// use DiemFramework::DiemAccount;
// fun main(account: signer) {
//     let account = &account;
//     DiemAccount::create_validator_operator_account(
//         account,
//         @{{bob}},
//         {{bob::auth_key}},
//         b"bob"
//     );
// }
// }
// // check: "Keep(ABORTED { code: 2,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::new_validator_operator_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 6,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::DiemAccount;
// use DiemFramework::XUS::XUS;
// fun main(account: signer) {
//     let account = &account;
//     DiemAccount::create_parent_vasp_account<XUS>(
//         account,
//         @{{bob}},
//         {{bob::auth_key}},
//         b"bob",
//         false
//     );
// }
// }
// // check: "Keep(ABORTED { code: 258,"

// //! new-transaction
// //! sender: blessed
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::new_parent_vasp_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 6,"

// //! new-transaction
// //! sender: blessed
// script {
// use DiemFramework::DiemAccount;
// use DiemFramework::XUS::XUS;
// fun main(account: signer) {
//     let account = &account;
//     DiemAccount::create_child_vasp_account<XUS>(
//         account,
//         @{{bob}},
//         {{bob::auth_key}},
//         false
//     );
// }
// }
// // check: "Keep(ABORTED { code: 771,"

// //! new-transaction
// //! sender: blessed
// //! type-args: 0x1::XUS::XUS
// //! args: 0, {{vasp}}, {{vasp::auth_key}}, b"vasp", true
// stdlib_script::AccountCreationScripts::create_parent_vasp_account
// // check: "Keep(EXECUTED)"

// //! new-transaction
// //! sender: vasp
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     Roles::new_child_vasp_role(account, account);
// }
// }
// // check: "Keep(ABORTED { code: 6,"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// fun main(account: signer) {
//     let account = &account;
//     assert(!Roles::can_hold_balance(account), 1);
// }
// }
// // check: "Keep(EXECUTED)"

// //! new-transaction
// //! sender: diemroot
// script {
// use DiemFramework::Roles;
// use Std::Signer;
// fun main(account: signer) {
//     let account = &account;
//     assert(!Roles::has_validator_role(account), 1);
//     assert(!Roles::has_validator_operator_role(account), 1);
//     assert(Roles::get_role_id(Signer::address_of(account)) == 0, 1);
// }
// }
// // check: "Keep(EXECUTED)"

// //! new-transaction
// //! sender: blessed
// script {
// use DiemFramework::Roles;
// use Std::Signer;
// fun main(account: signer) {
//     let account = &account;
//     assert(Roles::has_treasury_compliance_role(account), 0);
//     assert(Roles::get_role_id(Signer::address_of(account)) == 1, 1);
// }
// }
// // check: "Keep(EXECUTED)"

// //! new-transaction
// script {
// use DiemFramework::Roles;
// fun main(_account: signer) {
//     let _account = &_account;
//     let _ = Roles::get_role_id(@0x7); // does not exist, should abort
// }
// }
// // check: "Keep(ABORTED"
