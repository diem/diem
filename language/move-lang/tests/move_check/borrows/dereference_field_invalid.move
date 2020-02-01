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
        *s;
        *f;

        let f = id_mut(&mut s.f);
        *s;
        *f;
    }
}
