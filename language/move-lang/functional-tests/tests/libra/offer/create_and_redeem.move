//! account: alice
//! account: bob
//! account: carl

// Just a dummy module with a test resource
module M {
    resource struct T { b: bool }

    public fun create(): T {
        T { b: true }
    }

    public fun publish(account: &signer, t: T) {
        move_to(account,  t);
    }
}

//! new-transaction
//! sender: alice
script {
use {{default}}::M;
use 0x1::Offer;
use 0x1::Signer;

// Alice creates an offer for Bob that contains an M::T resource
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    Offer::create(account, M::create(), {{bob}});
    assert(Offer::exists_at<M::T>(sender), 77);
    assert(Offer::address_of<M::T>(sender) == {{bob}} , 78);
}
}

//! new-transaction
//! sender: carl
script {
use {{default}}::M;
use 0x1::Offer;

// Carl should *not* be able to claim Alice's offer for Bob
fun main(account: &signer) {
    M::publish(account, Offer::redeem(account, {{alice}}));
}
}
// check: "Keep(ABORTED { code: 7,"

//! new-transaction
//! sender: bob
script {
use {{default}}::M;
use 0x1::Offer;

// Bob should be able to claim Alice's offer for him
fun main(account: &signer) {
    // claimed successfully
    let redeemed: M::T = Offer::redeem(account, {{alice}});

    // offer should not longer exist
    assert(!Offer::exists_at<M::T>({{alice}}), 79);

    // create a new offer for Carl
    Offer::create(account, redeemed, {{carl}});
}
}

//! new-transaction
//! sender: bob
script {
use {{default}}::M;
use 0x1::Offer;

// Bob should be able to reclaim his own offer for Carl
fun main(account: &signer) {
    M::publish(account, Offer::redeem<M::T>(account, {{bob}}));
}
}
