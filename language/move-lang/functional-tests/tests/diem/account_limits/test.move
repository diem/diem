//! account: validatorvivian, 10000000XUS, 0, validator
//! account: bob, 100000000XUS, 0
//! account: alice, 100000000XUS, 0
//! account: otherblessed, 0XUS, 0
//! account: otherbob, 0XUS, 0, address

//! account: moneybags, 1000000000000XUS

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 10048

//! new-transaction
//! sender: moneybags
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, @{{otherblessed}}, 1, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap)
}
}
// TODO: fix
// chec: ABORTED
// chec: 10047

// ----- Blessed updates max_inflow for unhosted wallet

//! new-transaction
//! sender: blessed
script {
    use DiemFramework::AccountLimits;
    use DiemFramework::XDX::XDX;
    fun main(tc_account: signer) {
    let tc_account = &tc_account;
        AccountLimits::update_limits_definition<XDX>(tc_account, @DiemRoot, 2, 2, 0, 0);
    }
}

// check: "Keep(EXECUTED)"

// ------ try and mint to unhosted bob, but inflow is higher than total flow

//! new-transaction
//! sender: moneybags
script {
    use DiemFramework::XUS::XUS;
    use DiemFramework::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 3, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap)
    }
}

// TODO fix (should ABORT) - update unhosted //! account directive, and flow/balance updates for accounts
// check: "Keep(EXECUTED)"


// --- increase limits limits

//! new-transaction
//! sender: blessed
script {
    use DiemFramework::AccountLimits;
    use DiemFramework::XDX::XDX;
    fun main(tc_account: signer) {
    let tc_account = &tc_account;
        AccountLimits::update_limits_definition<XDX>(tc_account, @DiemRoot, 1000, 1000, 1000, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    // Since we directly wrote into this account using fake data store, we
    // don't actually know that the balance is greater than 0 in the
    // account limits code, but it is.
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: moneybags
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, @{{otherblessed}}, 2, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: otherblessed
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: moneybags
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 2, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
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
    use DiemFramework::XUS::XUS;
    use DiemFramework::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 100, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: moneybags
script {
    use DiemFramework::XUS::XUS;
    use DiemFramework::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// check: 9

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 101, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// chec: ABORTED
// check: 11

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{alice}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! new-transaction
//! sender: moneybags
script {
    use DiemFramework::XUS::XUS;
    use DiemFramework::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 1, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{otherbob}}, {{otherbob::auth_key}}, b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
//! check: "Keep(EXECUTED)"
