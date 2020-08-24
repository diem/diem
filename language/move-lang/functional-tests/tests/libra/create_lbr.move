//! account: bob, 0LBR, 0, address
//! account: charlie, 10000000Coin1
//! account: denise, 10000000Coin2

// create a parent VASP account for Bob
//! new-transaction
//! sender: blessed
script {
use 0x1::LBR::LBR;
use 0x1::LibraAccount;
fun main(lr_account: &signer) {
    let add_all_currencies = false;

    LibraAccount::create_parent_vasp_account<LBR>(
        lr_account,
        {{bob}},
        {{bob::auth_key}},
        x"A1",
        add_all_currencies,
    );
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
// Setup bob's account as a multi-currency account.
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
    LibraAccount::add_currency<Coin2>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: charlie
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 10000000, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap)
}
}
// check: EXECUTED

//! new-transaction
//! sender: denise
script {
use 0x1::Coin2::Coin2;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin2>(&with_cap, {{bob}}, 10000000, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap)
}
}
// check: EXECUTED

// Now create LBR in bob's account
//! new-transaction
//! sender: bob
script {
use 0x1::LBR::LBR;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let amount_lbr = 10;
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::staple_lbr(&with_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<LBR>({{bob}}) == 10, 77);
}
}
// check: EXECUTED

// Now unpack from the LBR into the constituent coins
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x1::LBR::LBR;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let amount_lbr = 10;
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::unstaple_lbr(&with_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<LBR>({{bob}}) == 0, 78);
}
}
// not: PreburnEvent
// not: BurnEvent
// check: EXECUTED
