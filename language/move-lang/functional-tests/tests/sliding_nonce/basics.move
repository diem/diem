//! account: bob, 100000000, 0, unhosted
//! account: alice, 100000000, 0, unhosted

// ****
// Account setup - bob is account with nonce resource and alice is a regular account
// ****

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: &signer) {
        SlidingNonce::publish(account);
        SlidingNonce::record_nonce_or_abort(account, 129);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: &signer) {
        SlidingNonce::record_nonce_or_abort(account, 1);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: &signer) {
        SlidingNonce::publish_nonce_resource(account, account);
    }
}
// check: "Keep(ABORTED { code: 4,"
