//! new-transaction
script {
    use 0x1::LibraConfig::{Self};
    fun main(account: &signer) {
        LibraConfig::initialize(account);
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 0

//! new-transaction
script {
    use 0x1::LibraConfig;
    fun main() {
        let _x = LibraConfig::get<u64>();
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 3

//! new-transaction
script {
    use 0x1::LibraConfig;
    fun main(account: &signer) {
        LibraConfig::set(account, 0);
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 3

//! new-transaction
script {
    use 0x1::LibraConfig::{Self};
    fun main(account: &signer) {
        LibraConfig::publish_new_config(account, 0);
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 0
