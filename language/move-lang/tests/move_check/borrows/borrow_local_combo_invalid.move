module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, s: S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        let x = &s;
        *f;
        *x;
    }

    t1(cond: bool, s: S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        let x = &mut s;
        *f;
        *x;
        *f;
    }

    t2(cond: bool, s: S) {
        let f;
        if (cond) f = &mut s.f else f = &mut s.g;
        let x = &s;
        *f;
        *x;
    }

    t3(cond: bool, s: S, other: &mut S) {
        let x;
        if (cond) x = &mut s else x = other;
        let y = &s;
        *y;
        *x;
        *y;
    }

    t4(cond: bool, s: S, other: &mut S) {
        let x;
        if (cond) x = &mut s else x = other;
        let y = &mut s;
        *y;
        *x;
    }

}
