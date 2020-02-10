module M {
    struct S { f: u64 }
    fun foo() {
        let s = S { false };
        let s = S { 0 };
    }
}
