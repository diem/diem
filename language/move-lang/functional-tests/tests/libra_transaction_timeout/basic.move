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
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let cap = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraTransactionTimeout::set_timeout(&cap, 0);
        Roles::restore_capability_to_privilege(account, cap);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
script {
    use 0x1::LibraTransactionTimeout;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let cap = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraTransactionTimeout::set_timeout(&cap, 0);
        Roles::restore_capability_to_privilege(account, cap);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: association
script {
    use 0x1::LibraTransactionTimeout;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let cap = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraTransactionTimeout::set_timeout(&cap, 86400000000);
        Roles::restore_capability_to_privilege(account, cap);
    }
}
