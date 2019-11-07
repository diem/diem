module M {
    struct S { f: u64 }
    foo() {
        let f;
        { f } = S { f: 0 };

        S f = S { f: 0 };
    }
}
