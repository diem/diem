script {
use 0x1::Offer;
fun main(account: &signer) {
    Offer::redeem<u64>(account, 0xA11CE);
    Offer::address_of<u64>(0xA11CE);
}
}
