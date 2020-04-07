//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator

//! sender: alice
module FooConfig {
    struct T {
        version: u64,
    }

    public fun new(version: u64): T {
        T { version: version }
    }
}

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
// Publish a new config item.
use 0x0::LibraConfig;
use {{alice}}::FooConfig;
fun main() {
    let config: FooConfig::T;

    config = FooConfig::new(0);
    LibraConfig::publish_new_config<FooConfig::T>(move config)
}
// Should trigger a reconfiguration
// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: association
// Cannot modify the value immediately.
use 0x0::LibraConfig;
use {{alice}}::FooConfig;
fun main() {
    let config: FooConfig::T;

    config = FooConfig::new(0);
    LibraConfig::set<FooConfig::T>(0x0::Transaction::sender(), move config)
}
// check: ABORTED
// check: 23

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
// Update the value after reconfiguration.
use 0x0::LibraConfig;
use {{alice}}::FooConfig;
fun main() {
    let config: FooConfig::T;

    config = FooConfig::new(0);
    LibraConfig::set<FooConfig::T>(0x0::Transaction::sender(), move config)
}
// Should trigger a reconfiguration
// check: NewEpochEvent
// check: EXECUTED
