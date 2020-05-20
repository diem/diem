//! account: bob, 100000000, 0, unhosted
//! account: alice, 100000000, 0, unhosted

// ****
// Account setup - bob is account with nonce resource and alice is a regular account
// ****

//! new-transaction
//! sender: bob
script {
    use 0x0::SlidingNonce;
    use 0x0::Transaction;

    fun main() {
        SlidingNonce::publish_nonce_resource_for_user();

        // 0-nonce is always allowed
        SlidingNonce::record_nonce_or_abort(0);
        SlidingNonce::record_nonce_or_abort(0);

        // Repeating nonce is not allowed
        SlidingNonce::record_nonce_or_abort(1);
        Transaction::assert(SlidingNonce::try_record_nonce(1) == 10003, 1);
        SlidingNonce::record_nonce_or_abort(2);

        // Can execute 1000 + 127 once(but not second time) and then 1000, because distance between them is <128
        SlidingNonce::record_nonce_or_abort(1000 + 127);
        Transaction::assert(SlidingNonce::try_record_nonce(1000 + 127) == 10003, 1);
        SlidingNonce::record_nonce_or_abort(1000);

        // Can execute 2000 + 128 but not 2000, because distance between them is <128
        SlidingNonce::record_nonce_or_abort(2000 + 128);
        Transaction::assert(SlidingNonce::try_record_nonce(2000) == 10001, 1);

        // Big jump is nonce is not allowed
        Transaction::assert(SlidingNonce::try_record_nonce(20000) == 10002, 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x0::SlidingNonce;

    fun main() {
        SlidingNonce::record_nonce_or_abort(0);
        SlidingNonce::record_nonce_or_abort(0);
    }
}
// check: EXECUTED
