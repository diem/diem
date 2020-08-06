//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator

// Changing the publishing option from Open to CustomScript
// Step 1: Make sure we can publish module at the beginning
module FooConfig {
    struct T {
        version: u64,
    }

    public fun new(version: u64): T {
        T { version: version }
    }
}

// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: libraroot
// Step 2: Change option to CustomModule
script {
use 0x1::LibraTransactionPublishingOption;

fun main(config: &signer) {
    LibraTransactionPublishingOption::set_open_module(config, false)
}
}

// check: EXECUTED
// check: NewEpochEvent

//! new-transaction
module FooConfig2 {
    struct T {
        version: u64,
    }

    public fun new(version: u64): T {
        T { version: version }
    }
}

// check: INVALID_MODULE_PUBLISHER
