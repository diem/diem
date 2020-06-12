//! account: bob
//! account: vasp, 0, 0, empty

//! new-transaction
//! sender: bob
script {
use 0x0::LibraAccount;
// not frozen
fun main() {
    assert(!LibraAccount::account_is_frozen({{bob}}), 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
// A special association privilege is needed for freezing an account
fun main(account: &signer) {
    LibraAccount::freeze_account(account, {{bob}});
}
}
// check: ABORTED
// check: 13

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
// Make sure we can freeze and unfreeze accounts.
fun main(account: &signer) {
    LibraAccount::freeze_account(account, {{bob}});
    assert(LibraAccount::account_is_frozen({{bob}}), 1);
    LibraAccount::unfreeze_account(account, {{bob}});
    assert(!LibraAccount::account_is_frozen({{bob}}), 2);
    LibraAccount::freeze_account(account, {{bob}});
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
fun main(account: &signer) {
    LibraAccount::unfreeze_account(account, {{bob}});
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
fun main(account: &signer) {
    LibraAccount::freeze_account(account, {{association}})
}
}
// check: ABORTED
// check: 14

// TODO: this can go away once //! account works
// create a parent VASPx
//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
fun main(association: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    LibraAccount::add_parent_vasp_role_from_association(
        association, {{vasp}}, x"A", x"B", pubkey
    )
}
}
// check: EXECUTED

// create a child VASP
//! new-transaction
//! sender: vasp
script {
use 0x0::LibraAccount;
use 0x0::LBR::LBR;
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    LibraAccount::create_child_vasp_account<LBR>(parent_vasp, 0xAA, dummy_auth_key_prefix, false);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
// Freezing a child account doesn't freeze the root, freezing the root
// doesn't freeze the child
fun main(account: &signer) {
    LibraAccount::freeze_account(account, 0xAA);
    assert(LibraAccount::account_is_frozen(0xAA), 3);
    assert(!LibraAccount::account_is_frozen({{vasp}}), 4);
    LibraAccount::unfreeze_account(account, 0xAA);
    assert(!LibraAccount::account_is_frozen(0xAA), 5);
    LibraAccount::freeze_account(account, {{vasp}});
    assert(LibraAccount::account_is_frozen({{vasp}}), 6);
    assert(!LibraAccount::account_is_frozen(0xAA), 7);
    LibraAccount::unfreeze_account(account, {{vasp}});
    assert(!LibraAccount::account_is_frozen({{vasp}}), 8);
}
}

// check: FreezeAccountEvent
// check: UnfreezeAccountEvent
// check: EXECUTED
