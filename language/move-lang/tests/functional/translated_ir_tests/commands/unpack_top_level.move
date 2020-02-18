module Test {
    struct T { b: bool }

    public fun new_t(): T {
        T { b: true }
    }

}

//! new-transaction
use {{default}}::Test;

fun main() {
    let t = Test::new_t();
    Test::T { b: _ } = t;
}

// check: error:
// check: Status(Failure)
