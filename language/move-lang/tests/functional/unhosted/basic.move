//! new-transaction
script {
    use 0x0::Unhosted;
    fun main() {
        Unhosted::publish_global_limits_definition();
    }
}
// check: ABORTED
// check: 0

//! new-transaction
script {
    use 0x0::Unhosted;
    fun main() {
        let _t = Unhosted::create();
    }
}
// check: ABORTED
// check: 10041

//! new-transaction
//! sender: association
script {
    use 0x0::Unhosted;
    use 0x0::Testnet;
    fun main() {
        Testnet::remove_testnet();
        let _t = Unhosted::create();
        Testnet::initialize();
    }
}
