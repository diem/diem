//! new-transaction
module Test {
    struct T { b: bool }

    public fun new_t(): T {
        T { b: true }
    }

}

//! new-transaction
script {
use {{default}}::Test;

fun main() {
    let t = Test::new_t();
    Test { b: _ } = t;
}
}
// check: "Unbound type" "in current scope"
