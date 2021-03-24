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

// check: "Keep(EXECUTED)"

// initially, any script is allowed
//! new-transaction
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: signer) {
    let account = &account;
    assert(DiemTransactionPublishingOption::is_script_allowed(account, &x""), 0);
}
}
// check: "Keep(EXECUTED)"

// turning off open scripts is a privileged operation
//! new-transaction
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: signer) {
    let account = &account;
    DiemTransactionPublishingOption::set_open_script(account)
}
}
// check: "Keep(ABORTED { code: 2,"

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: diemroot
// Step 2: Change option to CustomModule
script {
use 0x1::DiemTransactionPublishingOption;

fun main(config: signer) {
    let config = &config;
    DiemTransactionPublishingOption::set_open_module(config, false)
}
}

// check: "Keep(EXECUTED)"
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
