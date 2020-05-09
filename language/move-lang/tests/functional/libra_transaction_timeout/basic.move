//! new-transaction
script {
    use 0x0::LibraTransactionTimeout;
    fun main(account: &signer) {
        LibraTransactionTimeout::initialize(account);
    }
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x0::LibraTransactionTimeout;
    fun main() {
        LibraTransactionTimeout::set_timeout(0);
    }
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x0::LibraTransactionTimeout;
    fun main() {
        LibraTransactionTimeout::set_timeout(0);
    }
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: association
script {
    use 0x0::LibraTransactionTimeout;
    fun main() {
        LibraTransactionTimeout::set_timeout(86400000000);
    }
}
