// Test for running out of gas when updating the global state in the diem-vm.
// This is reported as VMStatus::Error(OUT_OF_GAS) instead of as
// VMStatus::ExecutionFailure(OUT_OF_GAS) because the error does not occur
// when running Move code and there is no associated location or offset.

//! account: default, 100000XUS
//! new-transaction
//! gas-price: 1
//! gas-currency: XUS
//! max-gas: 4
//! sender: default
script {
fun main() {
    let x: u64;
    let y: u64;

    x = 3;
    y = 5;

    assert(x + y == 8, 42);
}
}
// check: "ERROR { status_code: OUT_OF_GAS }"
// check: "gas_used: 4,"
// check: "Keep(OUT_OF_GAS)"
