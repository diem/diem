//! account: alice, 0, 0, 0Coin1

script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::Signer;

fun main(account: &signer) {
    let addr = Signer::address_of(account);
    let sender_balance = LibraAccount::balance<Coin1>(addr);
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, sender_balance, x"", x"");

    assert(LibraAccount::balance<Coin1>(addr) == 0, 42);

    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, sender_balance, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 1288,"
