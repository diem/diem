module 0x8675309::M {
    struct S has drop { f: u64 }
    fun foo() {
        S { f: 0, f: 0 };
    }
}
