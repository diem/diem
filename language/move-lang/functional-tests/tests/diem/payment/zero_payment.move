script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
use Std::Signer;

fun main(account: signer) {
    let account = &account;
    let addr = Signer::address_of(account);
    let old_balance = DiemAccount::balance<XUS>(addr);

    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, addr, 0, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);

    assert(DiemAccount::balance<XUS>(addr) == old_balance, 42);
}
}
// check: "Keep(ABORTED { code: 519,"
