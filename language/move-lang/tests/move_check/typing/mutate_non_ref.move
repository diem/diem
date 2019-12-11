module M {
    struct S { f: u64 }
    struct X { s: S }

    t0() {
        let u = 0;
        *u = 1;

        let s = S { f: 0 };
        *s = S { f: 0 };
        *s.f = 0;

        let s_ref = &mut S { f: 0 };
        *s_ref.f = 0;

        let x = X { s: *&s };
        *x.s = S { f: 0 };
        *x.s.f = 0;

        let x_ref = &mut X { s: *&s };
        *x_ref.s = S{ f: 0 };
        *x_ref.s.f = 0;

    }
}
