module A {
    struct S {value: u64}

    public fun t(changed: bool, s: &mut S) {
        if (changed) {
            foo(&mut move s.value);
        }
    }

    fun foo(x: &mut u64) {
        *x = *x + 1;
    }
}
