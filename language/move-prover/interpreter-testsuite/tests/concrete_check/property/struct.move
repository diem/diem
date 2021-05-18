module 0x2::A {
    struct S has drop {
        f1: bool,
        f2: u64,
    }

    #[test]
    public fun check_struct() {
        let a = S { f1: true, f2: 42 };
        let b = S { f1: true, f2: 0 };
        spec {
            assert a != b;
            assert a.f1 == b.f1;
            assert a == update_field(b, f2, 42);
            assert b != S { f1: false, f2: 0 };
        };
    }
}
