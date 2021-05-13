script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
use Std::Signer;

fun main(account: signer) {
    let account = &account;
    let addr = Signer::address_of(account);
    let balance = DiemAccount::balance<XUS>(addr);
    assert(balance > 10, 77);
}
}
