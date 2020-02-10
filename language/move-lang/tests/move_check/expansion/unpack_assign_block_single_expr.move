module M {
    struct S { f: u64 }
    fun foo() {
        S { 0 } = S { f: 0 };
    }
}
