//! account: bob
//! account: vasp, 0, 0, address
//! account: alice, 0, 0, address

//! new-transaction
//! sender: bob
script {
use 0x1::AccountFreezing;
// not frozen
fun main() {
    assert(!AccountFreezing::account_is_frozen({{bob}}), 0);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
use 0x1::AccountFreezing;
// A special association privilege is needed for freezing an account
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, {{bob}});
}
}
// check: "Keep(ABORTED { code: 258,"

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
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountFreezing::{Self};
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, {{diemroot}});
}
}
// check: "Keep(ABORTED { code: 775,"

// TODO: this can go away once //! account works
// create a parent VASPx
//! new-transaction
//! sender: blessed
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main(dr_account: &signer) {
    DiemAccount::create_parent_vasp_account<XUS>(
        dr_account,
        {{vasp}},
        {{vasp::auth_key}},
        x"A",
        true,
    );
}
}
// check: "Keep(EXECUTED)"

// create a child VASP
//! new-transaction
//! sender: vasp
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    DiemAccount::create_child_vasp_account<XUS>(parent_vasp, 0xAA, dummy_auth_key_prefix, false);
}
}
// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{vasp}});
    }
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::unfreeze_account(account, {{vasp}});
    }
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{diemroot}});
    }
}
// check: "Keep(ABORTED { code: 775,"

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{blessed}});
    }
}
// check: "Keep(ABORTED { code: 1031,"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::AccountFreezing;
    fun main() {
        assert(!AccountFreezing::account_is_frozen({{alice}}), 0);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
module Holder {
    resource struct Holder<T> { x: T }
    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Holder<T>{ x })
    }

    public fun get<T>(addr: address): T
    acquires Holder {
       let Holder<T> { x } = move_from<Holder<T>>(addr);
       x
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vasp
script {
use 0x1::DiemAccount;
use {{default}}::Holder;
fun main(account: &signer) {
    let cap = DiemAccount::extract_withdraw_capability(account);
    Holder::hold(account, cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{vasp}});
        assert(AccountFreezing::account_is_frozen({{vasp}}), 1);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{vasp}});
        assert(AccountFreezing::account_is_frozen({{vasp}}), 1);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{alice}}, {{alice::auth_key}}, b"bob", true
stdlib_script::create_parent_vasp_account
//! check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use {{default}}::Holder;
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(account: &signer) {
        let cap = Holder::get<DiemAccount::WithdrawCapability>({{vasp}});
        DiemAccount::pay_from<XUS>(&cap, {{alice}}, 0, x"", x"");
        Holder::hold(account, cap);
    }
}
// check: "Keep(ABORTED { code: 1281,"

//! new-transaction
//! sender: alice
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(account: &signer) {
        let cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&cap, {{vasp}}, 0, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 1281,"

//! new-transaction
//! sender: alice
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(account: &signer) {
        let cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&cap, {{vasp}}, 0, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 1281,"

//! new-transaction
//! sender: alice
script {
use 0x1::AccountFreezing;
fun main(account: &signer) {
    AccountFreezing::create(account);
}
}
// check: "Keep(ABORTED { code: 518,"

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountFreezing;
fun main(account: &signer) {
    AccountFreezing::freeze_account(account, 0x0);
}
}
// check: "Keep(ABORTED { code: 517,"

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountFreezing;
fun main(account: &signer) {
    AccountFreezing::unfreeze_account(account, 0x0);
}
}
// check: "Keep(ABORTED { code: 517,"
