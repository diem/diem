//! account: bob
//! account: vasp, 0, 0, vasp

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
// not frozen
fun main() {
    assert(!LibraAccount::account_is_frozen({{bob}}), 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x1::LibraAccount::{Self, AccountFreezing};
use 0x1::Roles;
// A special association privilege is needed for freezing an account
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AccountFreezing>(account);
    LibraAccount::freeze_account(account, &r, {{bob}});
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount::{Self, AccountFreezing, AccountUnfreezing};
use 0x1::Roles;
// Make sure we can freeze and unfreeze accounts.
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AccountFreezing>(account);
    let y = Roles::extract_privilege_to_capability<AccountUnfreezing>(account);
    LibraAccount::freeze_account(account, &r, {{bob}});
    assert(LibraAccount::account_is_frozen({{bob}}), 1);
    LibraAccount::unfreeze_account(account, &y, {{bob}});
    assert(!LibraAccount::account_is_frozen({{bob}}), 2);
    LibraAccount::freeze_account(account, &r, {{bob}});
    Roles::restore_capability_to_privilege(account, r);
    Roles::restore_capability_to_privilege(account, y);
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
use 0x1::LibraAccount::{Self, AccountUnfreezing};
use 0x1::Roles;
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AccountUnfreezing>(account);
    LibraAccount::unfreeze_account(account, &r, {{bob}});
    Roles::restore_capability_to_privilege(account, r);
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
use 0x1::LibraAccount::{Self, AccountFreezing};
use 0x1::Roles;
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AccountFreezing>(account);
    LibraAccount::freeze_account(account, &r, {{association}});
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: ABORTED
// check: 14

// TODO: this can go away once //! account works
// create a parent VASPx
//! new-transaction
//! sender: association
script {
use 0x1::LibraAccount;
use 0x1::Roles::{Self, AssociationRootRole};
fun main(association: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let r = Roles::extract_privilege_to_capability<AssociationRootRole>(association);
    LibraAccount::add_parent_vasp_role_from_association(
        &r, {{vasp}}, x"A", x"B", pubkey
    );
    Roles::restore_capability_to_privilege(association, r);
}
}
// check: EXECUTED

// create a child VASP
//! new-transaction
//! sender: vasp
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
use 0x1::Roles::{Self, ParentVASPRole};
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let r = Roles::extract_privilege_to_capability<ParentVASPRole>(parent_vasp);
    LibraAccount::create_child_vasp_account<LBR>(parent_vasp, &r, 0xAA, dummy_auth_key_prefix, false);
    Roles::restore_capability_to_privilege(parent_vasp, r);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount::{Self, AccountFreezing, AccountUnfreezing};
use 0x1::Roles;
// Freezing a child account doesn't freeze the root, freezing the root
// doesn't freeze the child
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AccountFreezing>(account);
    let y = Roles::extract_privilege_to_capability<AccountUnfreezing>(account);
    LibraAccount::freeze_account(account, &r, 0xAA);
    assert(LibraAccount::account_is_frozen(0xAA), 3);
    assert(!LibraAccount::account_is_frozen({{vasp}}), 4);
    LibraAccount::unfreeze_account(account, &y, 0xAA);
    assert(!LibraAccount::account_is_frozen(0xAA), 5);
    LibraAccount::freeze_account(account, &r, {{vasp}});
    assert(LibraAccount::account_is_frozen({{vasp}}), 6);
    assert(!LibraAccount::account_is_frozen(0xAA), 7);
    LibraAccount::unfreeze_account(account, &y, {{vasp}});
    assert(!LibraAccount::account_is_frozen({{vasp}}), 8);
    Roles::restore_capability_to_privilege(account, r);
    Roles::restore_capability_to_privilege(account, y);
}
}

// check: FreezeAccountEvent
// check: UnfreezeAccountEvent
// check: EXECUTED
