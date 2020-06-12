//! new-transaction
script {
    use 0x1::Unhosted;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let r = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        Unhosted::publish_global_limits_definition(account, &r);
        Roles::restore_capability_to_privilege(account, r)
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
