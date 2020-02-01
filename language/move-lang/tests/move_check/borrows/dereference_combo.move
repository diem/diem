module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, s: &mut S, other: &S,) {
        let f;
        if (cond) f = &s.f else f = &other.f;
        *s;
        *f;
        *s;
    }

    t1(cond: bool, s: &mut S) {
        let f;
        if (cond) f = &s.f else f = &s.g;
        *s;
        *f;
        *s;
    }

    t2(cond: bool, s: &mut S, other: &S) {
        let x: &S;
        if (cond) x = copy s else x = other;
        *s;
        *x;
    }

}
