// script 4, sender: bob
script {
use 0x1::M;
use 0x1::Offer;

// Bob should be able to reclaim his own offer for Carl
fun main(account: &signer) {
    M::publish(account, Offer::redeem<M::T>(account, 0xB0B));
}
}
