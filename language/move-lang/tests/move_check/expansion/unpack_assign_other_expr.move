module M {
    struct G {}
    struct S { f: u64 }
    foo() {
        let f;
        S ( f ) = S { f: 0 };

        let f;
        S f = S { f: 0 };

        G () = G {};
        G {{}} = G{};
    }
}
