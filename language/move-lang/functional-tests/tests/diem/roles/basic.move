//! account: bob, 0, 0, address
//! account: vasp, 0, 0, address
//! account: alice, 0, 0, address

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::grant_diem_root_role(account);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::grant_treasury_compliance_role(account, account);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_designated_dealer_role(account, account);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_designated_dealer_role(account, account);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DiemAccount::create_designated_dealer<XUS>(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob",
        false
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DiemAccount;
fun main(account: &signer) {
    DiemAccount::create_validator_account(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob",
    );
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_validator_role(account, account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DiemAccount;
fun main(account: &signer) {
    DiemAccount::create_validator_operator_account(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob"
    );
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_validator_operator_role(account, account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DiemAccount::create_parent_vasp_account<XUS>(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob",
        false
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_parent_vasp_role(account, account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DiemAccount::create_child_vasp_account<XUS>(
        account,
        {{bob}},
        {{bob::auth_key}},
        false
    );
}
}
// check: "Keep(ABORTED { code: 771,"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{vasp}}, {{vasp::auth_key}}, b"vasp", true
stdlib_script::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vasp
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_child_vasp_role(account, account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    assert(!Roles::can_hold_balance(account), 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
use 0x1::Roles;
use 0x1::Signer;
fun main(account: &signer) {
    assert(!Roles::has_validator_role(account), 1);
    assert(!Roles::has_validator_operator_role(account), 1);
    assert(Roles::get_role_id(Signer::address_of(account)) == 0, 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::Roles;
use 0x1::Signer;
fun main(account: &signer) {
    assert(Roles::has_treasury_compliance_role(account), 0);
    assert(Roles::get_role_id(Signer::address_of(account)) == 1, 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::Roles;
fun main(_account: &signer) {
    let _ = Roles::get_role_id(0x7); // does not exist, should abort
}
}
// check: "Keep(ABORTED"
