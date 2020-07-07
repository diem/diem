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
        true,
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
//! sender: libraroot
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{vasp}});
    }
}
// check: ABORTED
// check: "code: 2"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::unfreeze_account(account, {{vasp}});
    }
}
// check: ABORTED
// check: "code: 5"

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{libraroot}});
    }
}
// check: ABORTED
// check: "code: 3"

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{blessed}});
    }
}
// check: ABORTED
// check: "code: 4"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::initialize(account);
    }
}
// check: ABORTED
// check: "code: 0"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::AccountFreezing;
    fun main() {
        assert(!AccountFreezing::account_is_frozen({{alice}}), 0);
    }
}
// check: EXECUTED

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

//! new-transaction
//! sender: vasp
script {
use 0x1::LibraAccount;
use {{default}}::Holder;
fun main(account: &signer) {
    let cap = LibraAccount::extract_withdraw_capability(account);
    Holder::hold(account, cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{vasp}});
        assert(AccountFreezing::account_is_frozen({{vasp}}), 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountFreezing;
    fun main(account: &signer) {
        AccountFreezing::freeze_account(account, {{vasp}});
        assert(AccountFreezing::account_is_frozen({{vasp}}), 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{alice}}, {{alice::auth_key}}, b"bob", b"boburl", x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d", true
stdlib_script::create_parent_vasp_account
//! check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use {{default}}::Holder;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let cap = Holder::get<LibraAccount::WithdrawCapability>({{vasp}});
        LibraAccount::pay_from<Coin1>(&cap, {{alice}}, 0, x"", x"");
        Holder::hold(account, cap);
    }
}
// check: "Keep(ABORTED { code: 16,"

//! new-transaction
//! sender: alice
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&cap, {{vasp}}, 0, x"", x"");
        LibraAccount::restore_withdraw_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 16,"

//! new-transaction
//! sender: alice
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&cap, {{vasp}}, 0, x"", x"");
        LibraAccount::restore_withdraw_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 16,"