module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, s: &mut S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        freeze(s);
        *f;
    }

    t1(cond: bool, s: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut s.g;
        freeze(s);
        *f;
    }

    t2(cond: bool, s: &mut S, other: &mut S) {
        let x;
        if (cond) x = s else x = other;
        freeze(s);
        *x;
    }

}
