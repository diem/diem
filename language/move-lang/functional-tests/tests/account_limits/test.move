//! account: validatorvivian, 10000000Coin1, 0, validator
//! account: bob, 100000000Coin1, 0
//! account: alice, 100000000Coin1, 0
//! account: otherblessed, 0Coin1, 0
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

// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"


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
// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"

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
// check: "Keep(EXECUTED)"

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
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{otherbob}}, {{otherbob::auth_key}}, b"bob", true
stdlib_script::create_parent_vasp_account
//! check: "Keep(EXECUTED)"
