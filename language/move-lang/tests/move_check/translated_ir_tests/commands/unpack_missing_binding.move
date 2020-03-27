module Test {
    struct X { b: bool }
    struct T { i: u64, x: X, b: bool, y: u64 }

    public fun new_t(): T {
        let x = X { b: true };
        T { i: 0, x: move x, b: false, y: 0 }
    }

    public fun destroy_t(t: T): (u64, X, bool) {
        let T { i, x, b: flag } = t;
        (i, x, flag)
    }

}
