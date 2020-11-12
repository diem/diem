// Script 1, seder: alice
script {
use 0x1::Offer;
use 0x1::Signer;

// Create a self offer containing a u64
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    Offer::create(account, 7, 0xA11CE);
    assert(Offer::address_of<u64>(sender) == 0xA11CE , 100);
    assert(Offer::redeem(account, 0xA11CE) == 7, 101);
}
}
