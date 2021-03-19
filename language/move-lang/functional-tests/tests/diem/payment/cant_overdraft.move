//! account: alice, 0, 0, 0XUS

script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;
use 0x1::Signer;

fun main(account: signer) {
    let account = &account;
    let addr = Signer::address_of(account);
    let sender_balance = DiemAccount::balance<XUS>(addr);
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, {{alice}}, sender_balance, x"", x"");

    assert(DiemAccount::balance<XUS>(addr) == 0, 42);

    DiemAccount::pay_from<XUS>(&with_cap, {{alice}}, sender_balance, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 1288,"
