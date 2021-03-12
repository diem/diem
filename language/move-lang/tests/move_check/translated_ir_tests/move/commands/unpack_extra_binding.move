module 0x8675309::Test {
    struct X { b: bool }
    struct T { i: u64, x: X }

    public fun new_t(): T {
        let x = X { b: true };
        T { i: 0, x }
    }

    public fun destroy_t(t: T): (u64, X, bool) {
        let T { i, x, b: flag } = t;
        (i, x, flag)
    }

}
// check: NEGATIVE_STACK_SIZE_WITHIN_BLOCK
