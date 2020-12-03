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


//! new-transaction
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    assert(DiemTransactionPublishingOption::is_script_allowed(account, &x""), 0);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    DiemTransactionPublishingOption::set_open_script(account)
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    DiemTransactionPublishingOption::set_open_script(account)
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    let x = x"0000000000000000000000000000000000000000000000000000000000000001";
    DiemTransactionPublishingOption::add_to_script_allow_list(account, x);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
fun main() {
}
}
// check: "Discard(UNKNOWN_SCRIPT)"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    DiemTransactionPublishingOption::set_open_script(account)
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    let x = x"0000000000000000000000000000000000000000000000000000000000000001";
    DiemTransactionPublishingOption::add_to_script_allow_list(account, *&x);
    DiemTransactionPublishingOption::add_to_script_allow_list(account, x);
}
}
// check: "Keep(ABORTED { code: 263,"

//! new-transaction
//! sender: diemroot
script {
use 0x1::DiemTransactionPublishingOption;
fun main(account: &signer) {
    DiemTransactionPublishingOption::add_to_script_allow_list(account, x"");
}
}
// check: "Keep(ABORTED { code: 7,"

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: diemroot
// Step 2: Change option to CustomModule
script {
use 0x1::DiemTransactionPublishingOption;

fun main(config: &signer) {
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
