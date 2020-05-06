//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator

//! sender: alice
module FooConfig {
    use 0x0::LibraConfig;

    struct T {
        version: u64,
    }

    public fun new(version: u64) {
        LibraConfig::publish_new_config_with_delegate<T>(T { version: version }, {{alice}});
    }

    public fun claim() {
        LibraConfig::claim_delegated_modify_config<T>(LibraConfig::default_config_address());
    }

    public fun set(version: u64) {
        LibraConfig::set(
            T { version }
        )
    }
}

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: config
// Publish a new config item.
script {
use {{alice}}::FooConfig;
fun main() {
    FooConfig::new(0);
}
}
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: alice
// Update the value.
script {
use {{alice}}::FooConfig;
fun main() {
    FooConfig::claim();
    FooConfig::set(0);
}
}
// Should trigger a reconfiguration
// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 4

//! new-transaction
//! sender: config
script {
use {{alice}}::FooConfig;
fun main() {
    FooConfig::set(0);
}
}
// check: ABORT
