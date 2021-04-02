module 0x42::M {
    fun foo(x: u64): u64 {
        x
    }
    spec fun foo {
        requires x == 0;
    }
}
