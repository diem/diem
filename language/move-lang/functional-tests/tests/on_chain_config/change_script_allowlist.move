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
//! args: x"0100010000000000000000000000000a550c18",
// Step 2: Change option to CustomScript
script {
use 0x1::LibraVMConfig;

fun main(config: &signer, args: vector<u8>) {
    LibraVMConfig::set_publishing_option(config, args)
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
