module M {
    struct S { f: u64 }
    foo() {
        let f;
        S { f, f } = S { f: 0 };
    }
}
