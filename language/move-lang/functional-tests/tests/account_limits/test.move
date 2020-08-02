//! account: validatorvivian, 10000000Coin1, 0, validator
//! account: bob, 100000000Coin1, 0, unhosted
//! account: alice, 100000000Coin1, 0, unhosted
//! account: otherblessed, 0Coin1, 0, unhosted
//! account: otherbob, 0Coin1, 0, address

//! account: moneybags, 1000000000000Coin1

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 10048

//! new-transaction
//! sender: moneybags
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{otherblessed}}, 1, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap)
}
}
// TODO: fix
// chec: ABORTED
// chec: 10047

// ----- Blessed updates max_inflow for unhosted wallet

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::CoreAddresses;
    use 0x1::LBR::LBR;
    fun main(tc_account: &signer) {
        AccountLimits::update_limits_definition<LBR>(tc_account, CoreAddresses::LIBRA_ROOT_ADDRESS(), 2, 2, 0, 0);
    }
}

// check: EXECUTED

// ------ try and mint to unhosted bob, but inflow is higher than total flow

//! new-transaction
//! sender: moneybags
script {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 3, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap)
    }
}

// TODO fix (should ABORT) - update unhosted //! account directive, and flow/balance updates for accounts
// check: EXECUTED


// --- increase limits limits

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::CoreAddresses;
    use 0x1::LBR::LBR;
    fun main(tc_account: &signer) {
        AccountLimits::update_limits_definition<LBR>(tc_account, CoreAddresses::LIBRA_ROOT_ADDRESS(), 1000, 1000, 1000, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    // Since we directly wrote into this account using fake data store, we
    // don't actually know that the balance is greater than 0 in the
    // account limits code, but it is.
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: moneybags
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{otherblessed}}, 2, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: otherblessed
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: moneybags
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 2, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! block-prologue
//! proposer: validatorvivian
//! block-time: 40001

//! new-transaction
//! sender: moneybags
script {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 100, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: moneybags
script {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// check: 9

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 101, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// chec: ABORTED
// check: 11

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! new-transaction
//! sender: moneybags
script {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 1, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{otherbob}}, {{otherbob::auth_key}}, b"bob", b"boburl", x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d", true
stdlib_script::create_parent_vasp_account
//! check: EXECUTED

//! new-transaction
//! sender: otherbob
script {
use 0x1::VASP;
use 0x1::Coin1::Coin1;
fun main(account: &signer)  {
    VASP::add_account_limits<Coin1>(account);
}
}
// check: ABORTED
// check: "code: 5"

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: {{libraroot}}, 1, 1, 1, 1, 1
stdlib_script::update_account_limit_definition
// check: ABORTED
// check: "code: 3"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: {{libraroot}}, 1, 1, 1, 1, 1
stdlib_script::update_account_limit_definition
// check: EXECUTED

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: {{otherbob}}, 10, {{libraroot}}
stdlib_script::update_account_limit_window_info
// check: ABORTED
// check: "code: 3"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: {{otherbob}}, 10, {{otherbob}}
stdlib_script::update_account_limit_window_info
// check: ABORTED
// check: "code: 4"

//! new-transaction
//! sender: otherbob
script {
use 0x1::AccountLimits;
use 0x1::Coin1::Coin1;
fun main()  {
    assert(AccountLimits::limits_definition_address<Coin1>({{otherbob}}) == {{libraroot}}, 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: otherbob
script {
use 0x1::AccountLimits;
use 0x1::Coin1::Coin1;
fun main()  {
    assert(AccountLimits::has_limits_published<Coin1>({{libraroot}}), 2);
    assert(!AccountLimits::has_limits_published<Coin1>({{otherbob}}), 3);
}
}
// check: EXECUTED

//! new-transaction
module Holder {
    resource struct Hold<T> { x: T}
    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Hold<T> { x})
    }
}

//! new-transaction
//! sender: libraroot
script {
use 0x1::AccountLimits;
use {{default}}::Holder;
fun main(account: &signer)  {
    Holder::hold(account, AccountLimits::grant_mutation_capability(account));
}
}
// check: ABORTED
// check: "code: 0"
