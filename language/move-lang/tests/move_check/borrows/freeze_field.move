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
        *f;
        freeze(s);
    }

    t1(s: &mut S) {
        let f = &s.f;
        freeze(s);
        *f;
    }

}
