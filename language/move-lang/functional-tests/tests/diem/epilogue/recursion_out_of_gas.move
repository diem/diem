//! account: default, 50000XUS

module {{default}}::M {
    public fun rec(x: u64) {
        rec(x)
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! gas-price: 0
//! max-gas: 5000
//! sender: default
script {
use {{default}}::M;
fun main() {
    M::rec(3);
}
}
// check: CALL_STACK_OVERFLOW
