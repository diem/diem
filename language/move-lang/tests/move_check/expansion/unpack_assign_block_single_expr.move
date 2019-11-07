module M {
    struct S { f: u64 }
    foo() {
        S { 0 } = S { f: 0 };
    }
}
