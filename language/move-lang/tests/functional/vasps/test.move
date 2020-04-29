//! account: validatorvivian, 1000000, 0, validator
//! account: root, 10000000, 0, true
//! account: child1, 10000000, 0, true
//! account: child2, 10000000, 0, true
//! account: child3, 10000000, 0, true
//! account: child4, 10000000, 0, true

//! new-transaction
//! sender: root
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    VASP::apply_for_vasp_root_credential(x"AAA", x"BBB", x"CCC", pubkey);
}
}
// check: EXECUTED

//! new-transaction
//! sender: root
//! gas-price: 0
script {
use 0x0::VASP;
use 0x0::Transaction;
// Make sure `root` isn't considered a root vasp account yet since it
// hasn't been certified.
fun main() {
    Transaction::assert(!VASP::is_root_vasp({{root}}), 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
script {
use 0x0::VASP;
// can't publish child account resources pointing to the root until the
// root is certified.
fun main() {
    VASP::apply_for_child_vasp_credential({{root}});
}
}
// check: ABORTED
// check: 7001

//! new-transaction
//! sender: association
//! gas-price: 0
script {
use 0x0::VASP;
// Can't grant the VASP: does not have the VASP CreationPrivilege yet.
fun main() {
    VASP::grant_vasp({{root}});
}
}
// check: ABORTED
// check: 7000

//! new-transaction
//! sender: association
script {
use 0x0::VASP;
use 0x0::Association;
// Add the VASP privilege now
fun main() {
    Association::apply_for_privilege<VASP::CreationPrivilege>();
    Association::grant_privilege<VASP::CreationPrivilege>({{association}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::VASP;
use 0x0::Transaction;
// Once the root vasp is certified it now is considered a root vasp
// account.
fun main() {
    VASP::grant_vasp({{root}});
    Transaction::assert(VASP::is_root_vasp({{root}}), 1);
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// VASP child accounts
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: child1
//! gas-price: 0
script {
use 0x0::VASP;
// Can't publish to-be-child accounts until the VASP says they're allowing
// child accounts
fun main() {
    VASP::apply_for_child_vasp_credential({{root}});
}
}
// check: ABORTED
// check: 2005

//! new-transaction
//! sender: root
//! gas-price: 0
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    Transaction::assert(!VASP::allows_child_accounts({{root}}), 2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: root
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    VASP::allow_child_accounts();
}
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{root}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: child2
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{root}})
}
}
// check: EXECUTED

//! new-transaction
//! sender: child3
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{root}})
}
}
// check: EXECUTED

//! new-transaction
//! sender: child4
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{root}})
}
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_parent_capability();
}
}
// check: ABORTED
// check: 7002

//! new-transaction
//! sender: root
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    VASP::grant_child_account({{child1}});
    VASP::grant_child_account({{child2}});
    VASP::grant_child_account({{child3}});
    Transaction::assert(!VASP::is_parent_vasp({{child1}}), 3);
    Transaction::assert(VASP::is_child_vasp({{child1}}), 4);
    Transaction::assert(!VASP::is_parent_vasp({{child2}}), 5);
    Transaction::assert(VASP::is_child_vasp({{child2}}), 6);
    Transaction::assert(!VASP::is_parent_vasp({{child3}}), 7);
    Transaction::assert(VASP::is_child_vasp({{child3}}), 8);
}
}
// check: EXECUTED

//! new-transaction
//! sender: child1
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_parent_capability();
}
}

//! new-transaction
//! sender: root
script {
use 0x0::VASP;
fun main() {
    VASP::grant_parent_capability({{child1}});
}
}

//! new-transaction
//! sender: child2
script {
use 0x0::VASP;
// child2 isn't a parent account, so can't grant child accounts
fun main() {
    VASP::grant_child_account({{child4}});
}
}
// check: ABORTED
// check: 2005

//! new-transaction
//! sender: child1
script {
use 0x0::VASP;
use 0x0::Transaction;
// child1 is a parent account so it can grant other child accounts
fun main() {
    // We can grant other accounts
    VASP::grant_child_account({{child4}});
    Transaction::assert(VASP::is_child_vasp({{child1}}), 9);
    Transaction::assert(VASP::is_child_vasp({{child4}}), 10);
    Transaction::assert(!VASP::is_root_vasp({{child1}}), 11);
    Transaction::assert(!VASP::is_root_vasp({{child4}}), 12);
}
}
// check: EXECUTED

//! new-transaction
//! sender: child2
script {
use 0x0::VASP;
// Can't remove a parent capability from a child account that doesn't have
// it.
fun main() {
    VASP::remove_parent_capability({{child1}});
}
}
// check: MISSING_DATA

//! new-transaction
//! sender: root
script {
use 0x0::VASP;
// Decertification of child accounts can only be after the parent
// capability has been taken away.
fun main() {
    VASP::decertify_child_account({{child1}});
}
}
// check: ABORTED
// check: 7003

//! new-transaction
//! sender: child2
script {
use 0x0::VASP;
// The decertifying address must have a parent capability
fun main() {
    VASP::decertify_child_account({{child3}});
}
}
// check: ABORTED
// check: 2005

//! new-transaction
//! sender: child1
script {
use 0x0::VASP;
use 0x0::Transaction;
// Can decertify and recertify
fun main() {
    VASP::decertify_child_account({{child2}});
    Transaction::assert(!VASP::is_child_vasp({{child2}}), 12);
    VASP::recertify_child_account({{child2}});
}
}

//! new-transaction
//! sender: root
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    VASP::remove_parent_capability({{child1}});
    Transaction::assert(!VASP::is_parent_vasp({{child1}}), 13);
    Transaction::assert(VASP::is_root_child_vasp({{root}}, {{child1}}), 14);
    Transaction::assert(VASP::is_root_child_vasp({{root}}, {{child2}}), 15);
    Transaction::assert(VASP::is_root_child_vasp({{root}}, {{child3}}), 16);
    Transaction::assert(VASP::is_root_child_vasp({{root}}, {{child4}}), 17);
    Transaction::assert(!VASP::is_root_child_vasp({{child1}}, {{root}}), 18);
}
}
// check: EXECUTED

//! new-transaction
//! sender: root
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    VASP::decertify_child_account({{child1}});
    Transaction::assert(!VASP::is_child_vasp({{child1}}), 19);
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// VASP root ca cert renewal
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: root
script {
use 0x0::VASP;
// Only the association can update the CA cert
fun main() {
    VASP::update_ca_cert({{root}}, x"DDD");
}
}
// check: ABORTED
// check: 2005

//! new-transaction
//! sender: association
script {
use 0x0::VASP;
use 0x0::Transaction;
// Make sure the CA cert is updated.
fun main() {
    Transaction::assert(VASP::ca_cert({{root}}) == x"CCC", 20);
    VASP::update_ca_cert({{root}}, x"DDD");
    Transaction::assert(VASP::ca_cert({{root}}) == x"DDD", 21);
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// VASP root account expiration
///////////////////////////////////////////////////////////////////////////

//! new-transaction
script {
use 0x0::VASP;
use 0x0::Transaction;
// Makes sure the accounts are still valid
fun main() {
    Transaction::assert(VASP::is_child_vasp({{child2}}), 22);
    Transaction::assert(VASP::is_child_vasp({{child3}}), 23);
    Transaction::assert(VASP::is_child_vasp({{child4}}), 24);
}
}
// check: EXECUTED

//! block-prologue
//! proposer: validatorvivian
//! block-time: 31540000000001

//! new-transaction
//! expiration-time: 31550000
script {
use 0x0::VASP;
use 0x0::Transaction;
// The root accounts certification from the association has expired. The
// root is no longer valid, and the child accounts are no longer VASP
// accounts.
fun main() {
    Transaction::assert(!VASP::is_child_vasp({{child2}}), 25);
    Transaction::assert(!VASP::is_child_vasp({{child3}}), 26);
    Transaction::assert(!VASP::is_child_vasp({{child4}}), 27);
}
}
// check: EXECUTED

//! new-transaction
//! sender: root
//! expiration-time: 31550000
script {
use 0x0::VASP;
// Root vasp cannot recertify itself
fun main() {
    VASP::recertify_vasp({{root}});
}
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: association
//! expiration-time: 31550000
script {
use 0x0::VASP;
use 0x0::Transaction;
// Recertify the VASP. Now all of its children are good as well
fun main() {
    VASP::recertify_vasp({{root}});
    Transaction::assert(VASP::is_root_vasp({{root}}), 28);
    Transaction::assert(VASP::is_child_vasp({{child2}}), 29);
    Transaction::assert(VASP::is_child_vasp({{child3}}), 30);
    Transaction::assert(VASP::is_child_vasp({{child4}}), 31);
    Transaction::assert(!VASP::is_child_vasp({{child1}}), 32);
    Transaction::assert(!VASP::is_child_vasp({{root}}), 33);
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// VASP root account decertification
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: root
//! expiration-time: 31550000
script {
use 0x0::VASP;
// Try to decertify the VASP. Fails with 3 since sender isn't association.
fun main() {
    VASP::decertify_vasp({{root}});
}
}
// check: ABORTED
// check: 7000

//! new-transaction
//! sender: association
//! expiration-time: 31550000
script {
use 0x0::VASP;
use 0x0::Transaction;
// Revoke the VASP. Now all its children aren't vasp accounts either.
fun main() {
    VASP::decertify_vasp({{root}});
    Transaction::assert(!VASP::is_root_vasp({{root}}), 34);
    Transaction::assert(!VASP::is_child_vasp({{child2}}), 35);
    Transaction::assert(!VASP::is_child_vasp({{child3}}), 36);
    Transaction::assert(!VASP::is_child_vasp({{child4}}), 37);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
//! expiration-time: 31550000
script {
use 0x0::VASP;
// Once decertified it can no longer be certified
fun main() {
    VASP::grant_vasp({{root}});
}
}
// check: ABORTED
// check: 2004

//! new-transaction
//! sender: association
//! expiration-time: 31550000
script {
use 0x0::VASP;
use 0x0::Transaction;
// Recertify the VASP. Now it and all of its children are good as well
fun main() {
    VASP::recertify_vasp({{root}});
    Transaction::assert(VASP::is_root_vasp({{root}}), 38);
    Transaction::assert(VASP::is_child_vasp({{child2}}), 39);
    Transaction::assert(VASP::is_child_vasp({{child3}}), 40);
    Transaction::assert(VASP::is_child_vasp({{child4}}), 41);
    Transaction::assert(!VASP::is_child_vasp({{child1}}), 42);
    Transaction::assert(!VASP::is_child_vasp({{root}}), 43);
}
}
// check: EXECUTED
