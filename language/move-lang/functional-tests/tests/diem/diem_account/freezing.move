//! account: bob
//! account: vasp, 0, 0, address
//! account: alice, 0, 0, address

// Keep these tests until adding unit tests for the prologue and Diem Account around freezing
// We need to keep some of them to test for events as well

//! new-transaction
//! sender: blessed
script {
use DiemFramework::AccountFreezing;
// Make sure we can freeze and unfreeze accounts.
fun main(account: signer) {
    let account = &account;
    AccountFreezing::freeze_account(account, @{{bob}});
}
}

// check: FreezeAccountEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
fun main() { }
}
// check: SENDING_ACCOUNT_FROZEN

//! new-transaction
//! sender: blessed
script {
use DiemFramework::AccountFreezing::{Self};
fun main(account: signer) {
    let account = &account;
    AccountFreezing::unfreeze_account(account, @{{bob}});
}
}
// check: UnfreezeAccountEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
fun main() { }
}
// check: "Keep(EXECUTED)"

// TODO: this can go away once //! account works
// create a parent VASPx
//! new-transaction
//! sender: blessed
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(dr_account: signer) {
    let dr_account = &dr_account;
    DiemAccount::create_parent_vasp_account<XUS>(
        dr_account,
        @{{vasp}},
        {{vasp::auth_key}},
        x"0A",
        true,
    );
}
}
// check: "Keep(EXECUTED)"

// create a child VASP
//! new-transaction
//! sender: vasp
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
fun main(parent_vasp: signer) {
    let parent_vasp = &parent_vasp;
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    DiemAccount::create_child_vasp_account<XUS>(parent_vasp, @0xAA, dummy_auth_key_prefix, false);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::AccountFreezing;
// Freezing a child account doesn't freeze the root, freezing the root
// doesn't freeze the child
fun main(account: signer) {
    let account = &account;
    AccountFreezing::freeze_account(account, @0xAA);
    assert(AccountFreezing::account_is_frozen(@0xAA), 3);
    assert(!AccountFreezing::account_is_frozen(@{{vasp}}), 4);
    AccountFreezing::unfreeze_account(account, @0xAA);
    assert(!AccountFreezing::account_is_frozen(@0xAA), 5);
    AccountFreezing::freeze_account(account, @{{vasp}});
    assert(AccountFreezing::account_is_frozen(@{{vasp}}), 6);
    assert(!AccountFreezing::account_is_frozen(@0xAA), 7);
    AccountFreezing::unfreeze_account(account, @{{vasp}});
    assert(!AccountFreezing::account_is_frozen(@{{vasp}}), 8);
}
}

// check: FreezeAccountEvent
// check: UnfreezeAccountEvent
// check: "Keep(EXECUTED)"

//! new-transaction
module {{default}}::Holder {
    struct Holder<T> has key { x: T }
    public fun hold<T: store>(account: &signer, x: T) {
        move_to(account, Holder<T>{ x })
    }

    public fun get<T: store>(addr: address): T
    acquires Holder {
       let Holder<T> { x } = move_from<Holder<T>>(addr);
       x
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vasp
script {
use DiemFramework::DiemAccount;
use {{default}}::Holder;
fun main(account: signer) {
    let account = &account;
    let cap = DiemAccount::extract_withdraw_capability(account);
    Holder::hold(account, cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use DiemFramework::AccountFreezing;
    fun main(account: signer) {
        let account = &account;
        AccountFreezing::freeze_account(account, @{{vasp}});
        assert(AccountFreezing::account_is_frozen(@{{vasp}}), 1);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use DiemFramework::AccountFreezing;
    fun main(account: signer) {
        let account = &account;
        AccountFreezing::freeze_account(account, @{{vasp}});
        assert(AccountFreezing::account_is_frozen(@{{vasp}}), 1);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{alice}}, {{alice::auth_key}}, b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
//! check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use {{default}}::Holder;
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
        let account = &account;
        let cap = Holder::get<DiemAccount::WithdrawCapability>(@{{vasp}});
        DiemAccount::pay_from<XUS>(&cap, @{{alice}}, 0, x"", x"");
        Holder::hold(account, cap);
    }
}
// check: "Keep(ABORTED { code: 1281,"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
        let account = &account;
        let cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&cap, @{{vasp}}, 0, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 1281,"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
        let account = &account;
        let cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&cap, @{{vasp}}, 0, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 1281,"
