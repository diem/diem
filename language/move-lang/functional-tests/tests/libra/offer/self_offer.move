// Self offers are silly, but ok.

script {
use 0x1::Offer;
use 0x1::Signer;

// Create a self offer containing a u64
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    Offer::create(account, 7, {{default}});
    assert(Offer::address_of<u64>(sender) == {{default}} , 100);
}
}

//! new-transaction
script {
use 0x1::Offer;

// Claim the self offer
fun main(account: &signer) {
    assert(Offer::redeem(account, {{default}}) == 7, 101);
}
}
