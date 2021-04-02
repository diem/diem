module 0x42::M {
    fun foo(x: &mut u64) {
        *x = 0;
        return
    }
    spec fun foo {
        ensures x == 0;
    }
}
