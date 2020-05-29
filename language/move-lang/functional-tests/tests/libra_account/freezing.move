//! account: bob
//! account: vasp, 0, 0, empty

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
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::Transaction;
// Make sure we can freeze and unfreeze accounts.
fun main() {
    LibraAccount::freeze_account({{bob}});
    Transaction::assert(LibraAccount::account_is_frozen({{bob}}), 1);
    LibraAccount::unfreeze_account({{bob}});
    Transaction::assert(!LibraAccount::account_is_frozen({{bob}}), 2);
    LibraAccount::freeze_account({{bob}});
}
}

// check: FreezeAccountEvent
// check: UnfreezeAccountEvent
// check: FreezeAccountEvent
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
fun main() { }
}
// check: SENDING_ACCOUNT_FROZEN

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
fun main() {
    LibraAccount::unfreeze_account({{bob}});
}
}

// check: UnfreezeAccountEvent

//! new-transaction
//! sender: bob
script {
fun main() { }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
fun main() {
    LibraAccount::freeze_account({{association}})
}
}
// check: ABORTED
// check: 14

// create a chid VASP
//! new-transaction
//! sender: vasp
script {
use 0x0::LibraAccount;
use 0x0::LBR;
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    LibraAccount::create_child_vasp_account<LBR::T>(parent_vasp, 0xAA, dummy_auth_key_prefix, false);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::Transaction;
// Freezing a child account doesn't freeze the root, freezing the root
// doesn't freeze the child
fun main() {
    LibraAccount::freeze_account(0xAA);
    Transaction::assert(LibraAccount::account_is_frozen(0xAA), 3);
    Transaction::assert(!LibraAccount::account_is_frozen({{vasp}}), 4);
    LibraAccount::unfreeze_account(0xAA);
    Transaction::assert(!LibraAccount::account_is_frozen(0xAA), 5);
    LibraAccount::freeze_account({{vasp}});
    Transaction::assert(LibraAccount::account_is_frozen({{vasp}}), 6);
    Transaction::assert(!LibraAccount::account_is_frozen(0xAA), 7);
    LibraAccount::unfreeze_account({{vasp}});
    Transaction::assert(!LibraAccount::account_is_frozen({{vasp}}), 8);
}
}

// check: FreezeAccountEvent
// check: UnfreezeAccountEvent
// check: EXECUTED
