script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;
use 0x1::Signer;

fun main(account: &signer) {
    let addr = Signer::address_of(account);
    let balance = DiemAccount::balance<XUS>(addr);
    assert(balance > 10, 77);
}
}
