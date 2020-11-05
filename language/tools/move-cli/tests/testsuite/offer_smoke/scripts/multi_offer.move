// Script 1, sender: alice
script {
use 0x1::Offer;
fun main(account: &signer) {
    Offer::create(account, 0, 0xA11CE);
    Offer::create(account, 0, 0x4);
}
}
