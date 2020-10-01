//! account: alice, 1000000
//! account: bob, 1000000

//! sender: alice
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 514, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(false, 42);
}
}
// check: "Keep(ABORTED { code: 42,"


//! new-transaction
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

fun main() {
    assert(LibraAccount::balance<Coin1>({{bob}}) == 1000000, 43);
}
}
// check: "Keep(EXECUTED)"
