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

script {
use {{default}}::Test;

fun main() {
    assert(Test::t() == 100, 42);
}
}
