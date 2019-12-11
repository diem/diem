module M {
    struct G {}
    struct S { f: u64 }
    foo() {
        let f: u64;
        S ( f ) = S { f: 0 };

        let f: u64;
        S f = S { f: 0 };

        G () = G {};
        G {{}} = G{};
    }
}
