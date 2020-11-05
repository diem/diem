// script 3, sender: bob
script {
use 0x1::M;
use 0x1::Offer;

// Bob should be able to claim Alice's offer for him
fun main(account: &signer) {
    // claimed successfully
    let redeemed: M::T = Offer::redeem(account, 0xA11CE);

    // offer should not longer exist
    assert(!Offer::exists_at<M::T>(0xA11CE), 79);

    // create a new offer for Carl
    Offer::create(account, redeemed, 0xCA21);
}
}
