script {
use 0x1::Offer;
fun main(account: &signer) {
    Offer::redeem<u64>(account, {{default}});
}
}

//! new-transaction
script {
use 0x1::Offer;
fun main() {
    Offer::address_of<u64>({{default}});
}
}
