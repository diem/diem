
module M {
    fun foo(x: &mut u64) {
        (freeze<u64>(x): &u64);
        (freeze<vector<bool>>(&mut any()): &vector<bool>);

        (assert<>(true, 42): ());
        (assert(true && false, *x): ());
        (assert(true || false, (0u8 as u64)): ());
    }

    fun any<T>(): T {
        abort 0
    }
}
