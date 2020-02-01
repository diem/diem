module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, s: &mut S, other: &S) {
        let f;
        if (cond) f = &s.f else f = &other.f;
        freeze(s);
        *f;
    }

    t1(cond: bool, s: &mut S) {
        let f;
        if (cond) f = &s.f else f = &s.g;
        freeze(s);
        *f;
    }

    t2(cond: bool, s: &mut S, other: &S) {
        let x;
        if (cond) x = freeze(s) else x = other;
        freeze(s);
        *x;
    }

}
