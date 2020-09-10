module M {
    struct S { f: u64 }
    fun foo() {
        let f: u64;
        { f } = S { f: 0 };

        S f = S { f: 0 };
    }
}
