//! account: bob
//! account: vasp, 0, 0, address

//! new-transaction
//! sender: bob
script {
use 0x1::AccountFreezing;
// not frozen
fun main() {
    assert(!AccountFreezing::account_is_frozen({{bob}}), 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
script {
use 0x1::AccountFreezing;
// A special association privilege is needed for freezing an account
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, {{bob}});
}
}
// check: ABORTED
// check: 100

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountFreezing;
// Make sure we can freeze and unfreeze accounts.
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, {{bob}});
    assert(AccountFreezing::account_is_frozen({{bob}}), 1);
    AccountFreezing::unfreeze_account(account, {{bob}});
    assert(!AccountFreezing::account_is_frozen({{bob}}), 2);
    AccountFreezing::freeze_account(account, {{bob}});
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
use 0x1::AccountFreezing::{Self};
fun main(account: &signer) {
    AccountFreezing::unfreeze_account(account, {{bob}});
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
use 0x1::AccountFreezing::{Self};
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, {{libraroot}});
}
}
// check: ABORTED
// check: 14

// TODO: this can go away once //! account works
// create a parent VASPx
//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
fun main(lr_account: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    LibraAccount::create_parent_vasp_account<LBR>(
        lr_account,
        {{vasp}},
        {{vasp::auth_key}},
        x"A",
        x"B",
        pubkey,
        false,
    );
}
}
// check: EXECUTED

// create a child VASP
//! new-transaction
//! sender: vasp
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    LibraAccount::create_child_vasp_account<LBR>(parent_vasp, 0xAA, dummy_auth_key_prefix, false);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountFreezing;
// Freezing a child account doesn't freeze the root, freezing the root
// doesn't freeze the child
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, 0xAA);
    assert(AccountFreezing::account_is_frozen(0xAA), 3);
    assert(!AccountFreezing::account_is_frozen({{vasp}}), 4);
    AccountFreezing::unfreeze_account(account, 0xAA);
    assert(!AccountFreezing::account_is_frozen(0xAA), 5);
    AccountFreezing::freeze_account(account, {{vasp}});
    assert(AccountFreezing::account_is_frozen({{vasp}}), 6);
    assert(!AccountFreezing::account_is_frozen(0xAA), 7);
    AccountFreezing::unfreeze_account(account, {{vasp}});
    assert(!AccountFreezing::account_is_frozen({{vasp}}), 8);
}
}

// check: FreezeAccountEvent
// check: UnfreezeAccountEvent
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountFreezing;
use 0x1::VASP;
// Freezing a child account doesn't freeze the root, freezing the root
// freezes the child according to the VASP module.
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, {{vasp}});
    assert(AccountFreezing::account_is_frozen({{vasp}}), 9);
    assert(!AccountFreezing::account_is_frozen(0xAA), 10);
    assert(VASP::is_frozen({{vasp}}), 11);
    assert(VASP::is_frozen(0xAA), 12);

    AccountFreezing::unfreeze_account(account, {{vasp}});
    AccountFreezing::freeze_account(account, 0xAA);
    assert(!AccountFreezing::account_is_frozen({{vasp}}), 13);
    assert(AccountFreezing::account_is_frozen(0xAA), 14);
    assert(!VASP::is_frozen({{vasp}}), 15);
    assert(VASP::is_frozen(0xAA), 16);
}
}
