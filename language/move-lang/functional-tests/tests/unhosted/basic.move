//! new-transaction
script {
    use 0x1::Unhosted;
    fun main(account: &signer) {
        Unhosted::publish_global_limits_definition(account);
    }
}
// check: ABORTED
// check: 0

//! new-transaction
script {
    use 0x1::Unhosted;
    fun main() {
        let _t = Unhosted::create();
    }
}
// check: ABORTED
// check: 10041

//! new-transaction
//! sender: association
script {
    use 0x1::Unhosted;
    use 0x1::Testnet;
    fun main(account: &signer) {
        Testnet::remove_testnet(account);
        let _t = Unhosted::create();
        Testnet::initialize(account);
    }
}
