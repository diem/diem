module M {
    struct S { f: u64 }
    fun foo() {
        let f;
        S { f, f } = S { f: 0 };
    }
}
