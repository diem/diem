//! account: bob, 100000000
//! account: alice, 100000000

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

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: &signer) {
        SlidingNonce::record_nonce_or_abort(account, 1);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: &signer) {
        SlidingNonce::publish_nonce_resource(account, account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: &signer) {
        SlidingNonce::try_record_nonce(account, 1);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: &signer) {
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: &signer) {
        SlidingNonce::publish(account);
    }
}

//! new-transaction
//! sender: libraroot
//! execute-as: default
script {
    use 0x1::SlidingNonce;
    fun main(lr_account: &signer, default_account: &signer) {
        SlidingNonce::publish_nonce_resource(lr_account, default_account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: &signer) {
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: &signer) {
        SlidingNonce::publish(account);
    }
}

//! new-transaction
//! sender: libraroot
//! execute-as: default
script {
    use 0x1::SlidingNonce;
    fun main(lr_account: &signer, default_account: &signer) {
        SlidingNonce::publish_nonce_resource(lr_account, default_account);
    }
}
