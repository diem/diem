module Test {
    public fun t(): u64 {
        if (true) return 100;
        0
    }
}

//! new-transaction

use {{default}}::Test;

fun main() {
    0x0::Transaction::assert(Test::t() == 100, 42);
}
