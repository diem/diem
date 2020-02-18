module Test {
    resource struct X { b: bool }
    resource struct T { i: u64, x: X, b: bool }

    public fun new_x(): X {
        X { b: true }
    }

    public fun new_t(x: X): T {
        T { i: 0, x, b: false }
    }

    public fun destroy_x(x: X) {
        X { b: _ } = x;
    }

    public fun destroy_t(t: T): (u64, X, bool) {
        let T { i, x, b: flag } = t;
        (i, x, flag)
    }

}

//! new-transaction

use {{default}}::Test;

fun main() {
    let x = Test::new_x();
    let t = Test::new_t(x);
    let (_, x, _) = Test::destroy_t(t);
    Test::destroy_x(x);
}
