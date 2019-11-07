module M {
    struct T {}
    struct S { f: u64, g: u64 }
    foo() {
        let f = 0;
        let g = 1;
        T {};
        T { };
        S { f, g };
        S { f: 0, g: 0};
        S { g: 0, f };
        S { g, f: 0 };
    }
}
