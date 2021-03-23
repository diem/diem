//! account: alice, 10000XUS
//! account: bob, 10000XUS

// Wrong sender
//! new-transaction
//! sender: blessed
script {
use 0x1::SystemAdministrationScripts;
fun main(dr_account: signer) {
    SystemAdministrationScripts::set_gas_constants(
        dr_account,
        0,         // sliding nonce
         4,        // global_memory_per_byte_cost
         9,        // global_memory_per_byte_write_cost
         600,      // min_transaction_gas_units
         600,      // large_transaction_cutoff
         8,        // intrinsic_gas_per_byte
         4000000,  // maximum_number_of_gas_units
         0,        // min_price_per_gas_unit
         10000,    // max_price_per_gas_unit
         4096,     // max_transaction_size_in_bytes
         1000,     // gas_unit_scaling_factor
         800,      // default_account_size
    )
}
}
// check: "Keep(ABORTED { code: 2,"

// min gas price greater than max gas price
//! new-transaction
//! sender: diemroot
script {
use 0x1::SystemAdministrationScripts;
fun main(dr_account: signer) {
    SystemAdministrationScripts::set_gas_constants(
        dr_account,
        0,         // sliding nonce
         4,        // global_memory_per_byte_cost
         9,        // global_memory_per_byte_write_cost
         600,      // min_transaction_gas_units
         600,      // large_transaction_cutoff
         8,        // intrinsic_gas_per_byte
         4000000,  // maximum_number_of_gas_units
         10,       // min_price_per_gas_unit
         9,        // max_price_per_gas_unit
         4096,     // max_transaction_size_in_bytes
         1000,     // gas_unit_scaling_factor
         800,      // default_account_size
    )
}
}
// check: "Keep(ABORTED { code: 7,"

// min transaction txn gas units greater than max txn gas units
//! new-transaction
//! sender: diemroot
script {
use 0x1::SystemAdministrationScripts;
fun main(dr_account: signer) {
    SystemAdministrationScripts::set_gas_constants(
        dr_account,
        0,          // sliding nonce
         4,        // global_memory_per_byte_cost
         9,        // global_memory_per_byte_write_cost
         6000,     // min_transaction_gas_units
         600,      // large_transaction_cutoff
         8,        // intrinsic_gas_per_byte
         5999,     // maximum_number_of_gas_units
         0,        // min_price_per_gas_unit
         1000,     // max_price_per_gas_unit
         4096,     // max_transaction_size_in_bytes
         1000,     // gas_unit_scaling_factor
         800,      // default_account_size
    )
}
}
// check: "Keep(ABORTED { code: 7,"

// charge min gas of alice
//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: XUS
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"

// increase the min_transaction gas units and min_price_per_gas_unit
//! new-transaction
//! sender: diemroot
script {
use 0x1::SystemAdministrationScripts;
fun main(dr_account: signer) {
    SystemAdministrationScripts::set_gas_constants(
        dr_account,
        0,         // sliding nonce
         4,        // global_memory_per_byte_cost
         9,        // global_memory_per_byte_write_cost
         6000,     // min_transaction_gas_units
         600,      // large_transaction_cutoff
         8,        // intrinsic_gas_per_byte
         4000000,  // maximum_number_of_gas_units
         1,        // min_price_per_gas_unit
         10000,    // max_price_per_gas_unit
         4096,     // max_transaction_size_in_bytes
         1000,     // gas_unit_scaling_factor
         800,      // default_account_size
    )
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! gas-price: 1
//! gas-currency: XUS
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! gas-price: 1
//! gas-currency: XUS
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main() {
    // Alice processed before the bump in min transaction gas units so should have more money left
    assert(DiemAccount::balance<XUS>({{bob}}) < DiemAccount::balance<XUS>({{alice}}), 42);
}
}
// check: "Keep(EXECUTED)"

// Can't process a transaction now with a gas price of zero since the lower bound was also changed.
//! new-transaction
//! sender: bob
//! gas-price: 0
//! gas-currency: XUS
script {
fun main() {
}
}
// check: "Discard(GAS_UNIT_PRICE_BELOW_MIN_BOUND)"
