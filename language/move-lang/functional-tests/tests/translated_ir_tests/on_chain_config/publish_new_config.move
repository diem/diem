//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator

//! sender: alice
module FooConfig {
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::Roles;

    struct T {
        version: u64,
    }

    public fun new(account: &signer, version: u64) {
        let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
        LibraConfig::publish_new_config<T>(account, &r, T { version: version });
        Roles::restore_capability_to_privilege(account, r);
    }

    public fun set(account: &signer, version: u64) {
        LibraConfig::set(
            account,
            T { version }
        )
    }
}

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
// Publish a new config item.
script {
use {{alice}}::FooConfig;
fun main(account: &signer) {
    FooConfig::new(account, 0);
}
}
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
// Update the value.
script {
use {{alice}}::FooConfig;
fun main(account: &signer) {
    FooConfig::set(account, 0);
}
}
// Should trigger a reconfiguration
// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 4

//! new-transaction
//! sender: alice
script {
use {{alice}}::FooConfig;
fun main(account: &signer) {
    FooConfig::set(account, 0);
}
}
// check: ABORT
