
module M {
    fun foo(x: &u64) {
        (freeze<u64>(x): &mut u64);
        (freeze<vector<bool>>(&any()): &mut vector<bool>);

        (assert<>(42, true): ());
        (assert(true && false, *x): bool);
        assert(true || false, 0u8);
    }

    fun any<T>(): T {
        abort 0
    }
}
