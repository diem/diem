script {
use 0x1::Offer;
fun main(account: &signer) {
    Offer::create(account, 0, {{default}});
}
}

//! new-transaction
script {
use 0x1::Offer;
fun main(account: &signer) {
    Offer::create(account, 0, 0x4);
}
}
