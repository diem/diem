//! account: alice, 0, 0, address
//! account: bob, 0, 0, address
//! account: richie, 10Coin1, unhosted
//! account: sally, 10Coin2, unhosted

// create parent VASP accounts for alice and bob
//! new-transaction
//! sender: libraroot
script {
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LibraAccount;
fun main(lr_account: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let add_all_currencies = false;

    LibraAccount::create_parent_vasp_account<Coin1>(
        lr_account,
        {{alice}},
        {{alice::auth_key}},
        x"A1",
        x"A2",
        copy pubkey,
        add_all_currencies,
    );

    LibraAccount::create_parent_vasp_account<Coin2>(
        lr_account,
        {{bob}},
        {{bob::auth_key}},
        x"B1",
        x"B2",
        pubkey,
        add_all_currencies,
    );
}
}
// check: EXECUTED

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
// check: EXECUTED

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
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin2>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: EXECUTED

// Adding a bogus currency should abort
//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::add_currency<u64>(account);
}
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 16

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
// check: ABORTED
// check: 17

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
// check: EXECUTED

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
// check: EXECUTED
