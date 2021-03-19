//! account: bob, 100000000
//! account: alice, 100000000

// ****
// Account setup - bob is account with nonce resource and alice is a regular account
// ****

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: signer) {
    let account = &account;
        SlidingNonce::publish(account);
        SlidingNonce::record_nonce_or_abort(account, 129);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: signer) {
    let account = &account;
        SlidingNonce::record_nonce_or_abort(account, 1);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::SlidingNonce;

    fun main(account: signer) {
    let account = &account;
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: signer) {
    let account = &account;
        SlidingNonce::try_record_nonce(account, 1);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: signer) {
    let account = &account;
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: signer) {
    let account = &account;
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(default_account: signer) {
    let default_account = &default_account;
        SlidingNonce::publish(default_account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: signer) {
    let account = &account;
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(account: signer) {
    let account = &account;
        SlidingNonce::publish(account);
    }
}

//! new-transaction
script {
    use 0x1::SlidingNonce;
    fun main(default_account: signer) {
    let default_account = &default_account;
        SlidingNonce::publish(default_account);
    }
}
