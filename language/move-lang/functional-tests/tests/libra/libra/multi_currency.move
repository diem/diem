//! account: alice, 0, 0, address
//! account: bob, 0, 0, address
//! account: richie, 10Coin1
//! account: sally, 10Coin2

// create parent VASP accounts for alice and bob
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LibraAccount;
fun main(tc_account: &signer) {
    let add_all_currencies = false;

    LibraAccount::create_parent_vasp_account<Coin1>(
        tc_account,
        {{alice}},
        {{alice::auth_key}},
        x"A1",
        add_all_currencies,
    );

    LibraAccount::create_parent_vasp_account<Coin2>(
        tc_account,
        {{bob}},
        {{bob::auth_key}},
        x"B1",
        add_all_currencies,
    );
}
}
// check: "Keep(EXECUTED)"

// Give alice money from richie
//! new-transaction
//! sender: richie
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

// Give bob money from sally
//! new-transaction
//! sender: sally
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin2>(&with_cap, {{bob}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(EXECUTED)"

// Adding a bogus currency should abort
//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::add_currency<u64>(account);
}
}
// check: "Keep(ABORTED { code: 261,"

// Adding Coin1 a second time should fail with ADD_EXISTING_CURRENCY
//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(ABORTED { code: 3846,"

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<Coin1>({{alice}}) == 0, 0);
    assert(LibraAccount::balance<Coin1>({{bob}}) == 10, 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin2>(&with_cap, {{alice}}, 10, x"", x"");
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<Coin1>({{bob}}) == 0, 2);
    assert(LibraAccount::balance<Coin2>({{bob}}) == 0, 3);
    assert(LibraAccount::balance<Coin1>({{alice}}) == 10, 4);
    assert(LibraAccount::balance<Coin2>({{alice}}) == 10, 5);
}
}
// check: "Keep(EXECUTED)"
