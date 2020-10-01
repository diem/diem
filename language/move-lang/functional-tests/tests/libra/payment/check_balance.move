script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::Signer;

fun main(account: &signer) {
    let addr = Signer::address_of(account);
    let balance = LibraAccount::balance<Coin1>(addr);
    assert(balance > 10, 77);
}
}
