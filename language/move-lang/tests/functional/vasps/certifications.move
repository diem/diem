//! account: validatorvivian, 1000000, 0, validator
//! account: aroot, 10000000, 0, empty
//! account: achild, 10000000, 0, empty
//! account: broot, 10000000, 0, empty
//! account: bchild, 10000000, 0, empty
//! account: bob, 10000000, 0, unhosted

//! new-transaction
//! sender: aroot
script {
use 0x0::VASP;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    VASP::apply_for_vasp_root_credential(x"AAA", x"BBB", pubkey);
}
}
// check: EXECUTED

//! new-transaction
//! sender: broot
script {
use 0x0::VASP;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    VASP::apply_for_vasp_root_credential(x"AAA", x"BBB", pubkey);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::VASP;
use 0x0::Association;
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
    VASP::grant_vasp({{aroot}});
    VASP::grant_vasp({{broot}});
    Transaction::assert(VASP::is_root_vasp({{aroot}}), 1);
    Transaction::assert(VASP::is_root_vasp({{broot}}), 1);
}
}
// check: EXECUTED

//! new-transaction
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    Transaction::assert(!VASP::allows_child_accounts({{aroot}}), 0);
    Transaction::assert(!VASP::allows_child_accounts({{bob}}), 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: aroot
script {
use 0x0::VASP;
fun main() {
    VASP::allow_child_accounts();
}
}
// check: EXECUTED

//! new-transaction
//! sender: broot
script {
use 0x0::VASP;
fun main() {
    VASP::allow_child_accounts();
}
}
// check: EXECUTED

//! new-transaction
//! sender: achild
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{aroot}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: bchild
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{broot}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: aroot
script {
use 0x0::VASP;
fun main() {
    VASP::grant_child_account({{achild}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: broot
script {
use 0x0::VASP;
fun main() {
    VASP::grant_child_account({{bchild}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: achild
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_parent_capability();
}
}

//! new-transaction
//! sender: aroot
script {
use 0x0::VASP;
fun main() {
    VASP::grant_parent_capability({{achild}});
}
}

//! new-transaction
//! sender: association
script {
use 0x0::VASP;
fun main() {
    VASP::decertify_vasp({{aroot}});
}
}
// check: EXECUTED

//! block-prologue
//! proposer: validatorvivian
//! block-time: 1

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    0x0::Transaction::assert(VASP::is_root_vasp({{aroot}}), 1);
}
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: achild
script {
use 0x0::VASP;
fun main() {
    VASP::decertify_child_account({{bchild}});
}
}
// check: ABORTED
// check: 7001

//! new-transaction
//! sender: achild
script {
use 0x0::VASP;
fun main() {
    VASP::recertify_child_account({{bchild}});
}
}
// check: ABORTED
// check: 7001

//! new-transaction
//! sender: achild
script {
use 0x0::VASP;
fun main() {
    VASP::grant_child_account({{default}});
}
}
// check: ABORTED
// check: 7001

//! new-transaction
//! sender: broot
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{broot}});
}
}
// check: ABORTED
// check: 7002

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    let _ = VASP::root_vasp_address({{broot}});
}
}
// check: EXECUTED

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    let _ = VASP::root_vasp_address({{bob}});
}
}
// check: ABORTED
// check: 2

//! new-transaction
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    Transaction::assert(VASP::human_name({{broot}}) == VASP::human_name({{bchild}}), 3);
    VASP::human_name({{aroot}});
}
}
// check: ABORTED
// check: 2

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    VASP::human_name({{achild}});
}
}
// check: ABORTED
// check: 2

//! new-transaction
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    Transaction::assert(VASP::base_url({{broot}}) == VASP::base_url({{bchild}}), 3);
    VASP::base_url({{aroot}});
}
}
// check: ABORTED
// check: 2

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    VASP::base_url({{achild}});
}
}
// check: ABORTED
// check: 2

//! new-transaction
script {
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    Transaction::assert(VASP::expiration_date({{broot}}) == VASP::expiration_date({{bchild}}), 3);
    VASP::expiration_date({{aroot}});
}
}
// check: ABORTED
// check: 2

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    VASP::expiration_date({{achild}});
}
}
// check: ABORTED
// check: 2
