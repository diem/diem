module Test {
    public fun t(): u64 {
        let x;
        if (true) {
            return 100
        } else {
            x = 0;
        };
        x
    }
}

//! new-transaction

use {{default}}::Test;

fun main() {
    0x0::Transaction::assert(Test::t() == 100, 42);
}
