// script 2: sender: Carl
script {
use 0x1::M;
use Std::Offer;

// Carl should *not* be able to claim Alice's offer for Bob
fun main(account: signer) {
    M::publish(&account, Offer::redeem(&account, @0xA11CE));
}
}
