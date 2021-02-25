module M {
    struct S has drop { f: u64 }
    fun foo() {
        S { f: 0, f: 0 };
    }
}
