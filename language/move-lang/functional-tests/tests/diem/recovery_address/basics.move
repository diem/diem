//! account: parent1, 0, 0, address
//! account: child1, 0, 0, address
//! account: child2, 0, 0, address
//! account: parent2, 0, 0, address

//! account: vasp1, 0, 0, address
//! account: vasp2, 0, 0, address

// === Setup ===

// create parent VASP accounts for parent1 and 2
// create a parent VASP
//! new-transaction
//! sender: blessed
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
fun main(tc_account: signer) {
    let tc_account = &tc_account;
    let add_all_currencies = false;

    DiemAccount::create_parent_vasp_account<XUS>(
        tc_account,
        @{{parent1}},
        {{parent1::auth_key}},
        x"A1",
        add_all_currencies,
    );

    DiemAccount::create_parent_vasp_account<XUS>(
        tc_account,
        @{{parent2}},
        {{parent2::auth_key}},
        x"B1",
        add_all_currencies,
    );

}
}
// check: "Keep(EXECUTED)"

// create two children for parent1
//! new-transaction
//! sender: parent1
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    DiemAccount::create_child_vasp_account<XUS>(account, @{{child1}}, {{child1::auth_key}}, false);
    DiemAccount::create_child_vasp_account<XUS>(account, @{{child2}}, {{child2::auth_key}}, false)
}
}
// check: "Keep(EXECUTED)"

// === Intended usage ===

// make child1 a recovery address
//! new-transaction
//! sender: child1
script {
use DiemFramework::DiemAccount;
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    RecoveryAddress::publish(account, DiemAccount::extract_key_rotation_capability(account))
}
}
// check: "Keep(EXECUTED)"

// delegate parent1's key to child1
//! new-transaction
//! sender: parent1
script {
use DiemFramework::DiemAccount;
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    RecoveryAddress::add_rotation_capability(
        DiemAccount::extract_key_rotation_capability(account), @{{child1}}
    );
}
}
// check: "Keep(EXECUTED)"

// ==== Abort cases ===

// delegating parent2's key to child1 should abort because they are different VASPs
//! new-transaction
//! sender: parent2
script {
use DiemFramework::DiemAccount;
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    RecoveryAddress::add_rotation_capability(
        DiemAccount::extract_key_rotation_capability(account), @{{child1}}
    )
}
}
// check: "ABORTED { code: 775,"

// delegating parent2's key to an account without a RecoveryAddress resource should abort
//! new-transaction
//! sender: parent2
script {
use DiemFramework::DiemAccount;
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    RecoveryAddress::add_rotation_capability(
        DiemAccount::extract_key_rotation_capability(account), @0x3333
    )
}
}
// check: "ABORTED { code: 1285,"

// trying to recover an account that hasn't delegated its KeyRotationCapability to a recovery
// address should abort
//! new-transaction
//! sender: child2
script {
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    let dummy_auth_key = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    RecoveryAddress::rotate_authentication_key(account, @{{child1}}, @{{child2}}, dummy_auth_key);
}
}
// check: "ABORTED { code: 1031,"

// trying to recover from an account without a RecoveryAddress resource should abort
//! new-transaction
//! sender: child1
script {
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    let dummy_auth_key = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    RecoveryAddress::rotate_authentication_key(account, @{{child2}}, @{{child1}}, dummy_auth_key);
}
}
// check: "ABORTED { code: 1285,"


// parent1 shouldn't be able to rotate child1's address
//! new-transaction
//! sender: parent1
script {
use DiemFramework::RecoveryAddress;
fun main(account: signer) {
    let account = &account;
    let dummy_auth_key = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    RecoveryAddress::rotate_authentication_key(account, @{{child1}}, @{{child1}}, dummy_auth_key);
}
}
// check: "ABORTED { code: 519,"

// A non-vasp can't create a recovery address
//! new-transaction
//! sender: blessed
script {
use DiemFramework::RecoveryAddress;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    RecoveryAddress::publish(account, DiemAccount::extract_key_rotation_capability(account))
}
}
// check: "ABORTED { code: 7,"

//! new-transaction
module {{default}}::Holder {
    struct Holder<T> has key { x: T }
    public fun hold<T: store>(account: &signer, x: T) {
        move_to(account, Holder<T> { x });
    }

    public fun get<T: store>(addr: address): T
    acquires Holder {
        let Holder<T>{ x } = move_from<Holder<T>>(addr);
        x
    }
}

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{vasp1}}, {{vasp1::auth_key}}, b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{vasp2}}, {{vasp2::auth_key}}, b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vasp1
script {
use {{default}}::Holder;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    Holder::hold(account, DiemAccount::extract_key_rotation_capability(account));
}
}
// check: "Keep(EXECUTED)"

// Try to create a recovery address with an invalid key rotation capability
//! new-transaction
//! sender: vasp2
script {
use DiemFramework::RecoveryAddress;
use {{default}}::Holder;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    let cap = Holder::get<DiemAccount::KeyRotationCapability>(@{{vasp1}});
    RecoveryAddress::publish(account, cap);
}
}
// check: "ABORTED { code: 263,"
