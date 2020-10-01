script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::Signer;

fun main(account: &signer) {
    let addr = Signer::address_of(account);
    let old_balance = LibraAccount::balance<Coin1>(addr);

    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, addr, 0, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);

    assert(LibraAccount::balance<Coin1>(addr) == old_balance, 42);
}
}
// check: "Keep(ABORTED { code: 519,"
