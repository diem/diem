module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = &s.f else f = &s.g;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    t1(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    t2(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = s else f = other;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    t3(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = id_mut(s) else f = other;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    t4(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f = &s.f;
        *f;
        if (cond) *s = S { f: 0, g: 0 };
        s;
    }

}
