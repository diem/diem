module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, s: S, other: &S) {
        let f;
        if (cond) f = &s.f else f = &other.f;
        let x = &mut s;
        *f;
        *x;
    }

    t1(cond: bool, s: S) {
        let f;
        if (cond) f = &s.f else f = &s.g;
        let x = &mut s;
        *f;
        *x;
    }

    t2(cond: bool, s: S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        let x = &mut s;
        *f;
        *x;
    }

    t3(cond: bool, s: S, other: &S) {
        let x;
        if (cond) x = &s else x = other;
        let y = &s;
        *x;
        *y;
    }

    t4(cond: bool, s: S, other: &mut S) {
        let x;
        if (cond) x = &mut s else x = other;
        let y = &mut s;
        *x;
        *y;
    }
}
