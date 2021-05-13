//! account: alice, 1000000
//! account: bob, 1000000

//! sender: alice
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;

fun main(account: signer) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, @{{bob}}, 514, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
    assert(false, 42);
}
}
// check: "Keep(ABORTED { code: 42,"


//! new-transaction
script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;

fun main() {
    assert(DiemAccount::balance<XUS>(@{{bob}}) == 1000000, 43);
}
}
// check: "Keep(EXECUTED)"
