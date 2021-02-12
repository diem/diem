script {
use 0x1::DiemAccount;

fun test<Token>(account: &signer) {
    let withdraw_cap = DiemAccount::extract_withdraw_capability(account);
    borrow_global<DiemAccount::DiemAccount>(0x1);
    move_to(account, withdraw_cap);
}
}
