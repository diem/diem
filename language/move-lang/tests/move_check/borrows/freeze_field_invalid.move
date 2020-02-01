module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(s: &mut S) {
        let f = &mut s.f;
        freeze(s);
        *f;
    }

    t1(s: &mut S) {
        let f = &s.f;
        let g = &mut s.f;
        freeze(s);
        *f;
        *g;
    }
}
