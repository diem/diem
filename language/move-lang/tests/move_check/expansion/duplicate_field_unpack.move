module 0x8675309::M {
    struct S { f: u64 }
    fun foo() {
        let S { f, f } = S { f: 0 };
    }
}
