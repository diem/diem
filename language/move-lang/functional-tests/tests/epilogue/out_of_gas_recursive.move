module OddOrEven {
    public fun is_odd(x: u64): bool { if (x == 0) false else is_even(x - 1) }
    public fun is_even(x: u64): bool { if (x == 0) true else is_odd(x - 1) }
}

//! new-transaction
//! max-gas: 600
script {
use {{default}}::OddOrEven;
fun main() {
    OddOrEven::is_odd(1001);
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS, location: a4a46d1b1421502568a4a6ac326d7250::OddOrEven,"
// check: "gas_used: 600,"
// check: "Keep(OUT_OF_GAS)"
