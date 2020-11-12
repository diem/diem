// 1
script {
use 0x1::M;
use 0x1::Offer;
use 0x1::Signer;

// Alice creates an offer for Bob that contains an M::T resource
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    Offer::create(account, M::create(), 0xB0B);
    assert(Offer::exists_at<M::T>(sender), 77);
    assert(Offer::address_of<M::T>(sender) == 0xB0B , 78);
}
}
