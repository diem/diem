//! account: bob
//! account: vasp, 0, 0,true
//! account: childvasp, 0, 0, true

//! new-transaction
//! sender: bob
script {
use 0x0::Transaction;
use 0x0::LibraAccount;
// not frozen
fun main() {
    Transaction::assert(!LibraAccount::account_is_frozen({{bob}}), 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
// A special association privilege is needed for freezing an account
fun main() {
    LibraAccount::freeze_account({{bob}});
}
}
// check: ABORTED
// check: 13

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
use 0x0::Association;
use 0x0::Transaction;
// Make sure we can freeze and unfreeze accounts.
fun main() {
    Association::apply_for_privilege<LibraAccount::FreezingPrivilege>();
    Association::grant_privilege<LibraAccount::FreezingPrivilege>({{association}});
    LibraAccount::freeze_account({{bob}});
    Transaction::assert(LibraAccount::account_is_frozen({{bob}}), 1);
    LibraAccount::unfreeze_account({{bob}});
    Transaction::assert(!LibraAccount::account_is_frozen({{bob}}), 2);
    LibraAccount::freeze_account({{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
fun main() { }
}
// check: SENDING_ACCOUNT_FROZEN

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
fun main() {
    LibraAccount::unfreeze_account({{bob}});
}
}

//! new-transaction
//! sender: bob
script {
fun main() { }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
fun main() {
    LibraAccount::freeze_account({{association}})
}
}
// check: ABORTED
// check: 14

//! new-transaction
//! sender: vasp
//! gas-price: 0
script {
use 0x0::VASP;
use 0x0::LCS;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    VASP::apply_for_vasp_root_credential(
        LCS::to_bytes<address>(&0xAAA),
        LCS::to_bytes<address>(&0xBBB),
        LCS::to_bytes<address>(&0xCCC),
        pubkey,
    );
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
    VASP::grant_vasp({{vasp}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: vasp
//! gas-price: 0
script {
fun main() {
    0x0::VASP::allow_child_accounts();
}
}
// check: EXECUTED

//! new-transaction
//! sender: childvasp
//! gas-price: 0
script {
fun main() {
    0x0::VASP::apply_for_child_vasp_credential({{vasp}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: vasp
//! gas-price: 0
script {
fun main() {
    0x0::VASP::grant_child_account({{childvasp}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
use 0x0::Transaction;
// Freezing a child account doesn't freeze the root, freezing the root
// doesn't freeze the child
fun main() {
    LibraAccount::freeze_account({{childvasp}});
    Transaction::assert(LibraAccount::account_is_frozen({{childvasp}}), 3);
    Transaction::assert(!LibraAccount::account_is_frozen({{vasp}}), 4);
    LibraAccount::unfreeze_account({{childvasp}});
    Transaction::assert(!LibraAccount::account_is_frozen({{childvasp}}), 5);
    LibraAccount::freeze_account({{vasp}});
    Transaction::assert(LibraAccount::account_is_frozen({{vasp}}), 6);
    Transaction::assert(!LibraAccount::account_is_frozen({{childvasp}}), 7);
    LibraAccount::unfreeze_account({{vasp}});
    Transaction::assert(!LibraAccount::account_is_frozen({{vasp}}), 8);
}
}
// check: EXECUTED
