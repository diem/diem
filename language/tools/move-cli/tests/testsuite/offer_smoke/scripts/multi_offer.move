// Script 1, sender: alice
script {
use Std::Offer;
fun main(account: signer) {
    Offer::create(&account, 0, @0xA11CE);
    Offer::create(&account, 0, @0x4);
}
}
