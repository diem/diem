//! account: bob, 0, 0, address
//! account: vasp, 0, 0, address
//! account: alice, 0, 0, address

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::grant_libra_root_role(account);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::grant_treasury_compliance_role(account, account);
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_designated_dealer_role(account, account);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_designated_dealer_role(account, account);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::create_designated_dealer<Coin1>(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob",
        b"boburl",
        x"",
        false
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::create_validator_account(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob",
    );
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
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
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::create_validator_operator_account(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob"
    );
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_validator_operator_role(account, account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::create_parent_vasp_account<Coin1>(
        account,
        {{bob}},
        {{bob::auth_key}},
        b"bob",
        b"boburl",
        x"",
        false
    );
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    Roles::new_parent_vasp_role(account, account);
}
}
// check: "Keep(ABORTED { code: 6,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::create_child_vasp_account<Coin1>(
        account,
        {{bob}},
        {{bob::auth_key}},
        false
    );
}
}
// check: "Keep(ABORTED { code: 771,"

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: {{vasp}}, {{vasp::auth_key}}, true
stdlib_script::create_testing_account
// check: EXECUTED

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
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    assert(!Roles::needs_account_limits(account), 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
script {
use 0x1::Roles;
fun main(account: &signer) {
    assert(!Roles::has_validator_role(account), 1);
    assert(!Roles::has_validator_operator_role(account), 1);
}
}
// check: EXECUTED
