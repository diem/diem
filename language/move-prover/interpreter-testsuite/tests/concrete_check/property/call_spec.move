module 0x2::A {
    struct S1 has copy, drop {
        f1: bool,
        f2: u64,
    }

    struct S2<T: copy> has copy, drop {
        f1: bool,
        f2: T,
    }

    spec fun spec_plain(s: S1): u64 {
        s.f2
    }

    spec fun spec_generic<T: copy>(s: S2<T>): T {
        s.f2
    }

    #[test]
    public fun check_call_spec() {
        let s1 = S1 { f1: true, f2: 42 };
        let s2 = S2 { f1: false, f2: s1 };
        let p1 = &s2.f2;
        let p2 = &s2;
        spec {
            assert spec_plain(p1) == 42;
            assert spec_generic(p2) == s1;
        };
    }
}
