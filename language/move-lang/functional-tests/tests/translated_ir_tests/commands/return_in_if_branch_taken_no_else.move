module Test {
    public fun t(): u64 {
        if (true) return 100;
        0
    }
}

//! new-transaction

script {
use {{default}}::Test;

fun main() {
    assert(Test::t() == 100, 42);
}
}
