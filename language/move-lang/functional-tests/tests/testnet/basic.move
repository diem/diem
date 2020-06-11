//! new-transaction
script {
    use 0x1::Testnet;
    fun main(account: &signer) {
        Testnet::initialize(account);
    }
}
// check: ABORTED
// check: 0

//! new-transaction
//! sender: association
script {
    use 0x1::Testnet;
    fun main(account: &signer) {
        Testnet::remove_testnet(account);
    }
}

//! new-transaction
script {
    use 0x1::Testnet;
    fun main(account: &signer) {
        Testnet::remove_testnet(account);
    }
}
// check: ABORTED
// check: 0
