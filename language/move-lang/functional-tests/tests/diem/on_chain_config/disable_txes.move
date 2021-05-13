//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

// disable txes from all accounts except DiemRoot
//! new-transaction
//! sender: diemroot
script {
use DiemFramework::DiemTransactionPublishingOption;
fun main(account: signer) {
    let account = &account;
    DiemTransactionPublishingOption::halt_all_transactions(account);
}
}
// check: "Keep(EXECUTED)"

// sending custom script from normal account fails
//! new-transaction
script {
fun main() {
}
}
// check: "Discard(UNKNOWN_SCRIPT)"

// sending allowlisted script from normal account fails
//! new-transaction
//! args: b""
stdlib_script::AccountAdministrationScripts::rotate_authentication_key
// check: "Discard(UNKNOWN_SCRIPT)"

//! block-prologue
//! proposer: vivian
//! block-time: 3

// re-enable. this also tests that sending from DiemRoot succeeds
//! new-transaction
//! sender: diemroot
script {
use DiemFramework::DiemTransactionPublishingOption;
fun main(account: signer) {
    let account = &account;
    DiemTransactionPublishingOption::resume_transactions(account);
}
}
// check: "Keep(EXECUTED)"

// sending from normal account succeeds again
//! new-transaction
//! args: b""
stdlib_script::AccountAdministrationScripts::rotate_authentication_key
// check: "Keep(ABORTED { code: 2055,"

// A normal account has insufficient privs to halt transactions
//! new-transaction
script {
use DiemFramework::DiemTransactionPublishingOption;
fun main(account: signer) {
    let account = &account;
    DiemTransactionPublishingOption::halt_all_transactions(account);
}
}
// check: "Keep(ABORTED { code: 2,"
