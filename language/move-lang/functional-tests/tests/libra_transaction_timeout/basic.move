//! new-transaction
script {
    use 0x1::LibraTransactionTimeout;
    fun main(account: &signer) {
        LibraTransactionTimeout::initialize(account);
    }
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x1::LibraTransactionTimeout;
    fun main(account: &signer) {
        LibraTransactionTimeout::set_timeout(account, 0);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
script {
    use 0x1::LibraTransactionTimeout;
    fun main(account: &signer) {
        LibraTransactionTimeout::set_timeout(account, 0);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraTransactionTimeout;
    fun main(account: &signer) {
        LibraTransactionTimeout::set_timeout(account, 86400000000);
    }
}
