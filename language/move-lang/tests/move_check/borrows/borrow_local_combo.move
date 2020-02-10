module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(cond: bool, s: S, other: &S) {
        let f;
        if (cond) f = &s.f else f = &other.f;
        let x = &mut s;
        *f;
        *x;
    }

    fun t1(cond: bool, s: S) {
        let f;
        if (cond) f = &s.f else f = &s.g;
        let x = &mut s;
        *f;
        *x;
    }

    fun t2(cond: bool, s: S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        let x = &mut s;
        *f;
        *x;
    }

    fun t3(cond: bool, s: S, other: &S) {
        let x;
        if (cond) x = &s else x = other;
        let y = &s;
        *x;
        *y;
    }

    fun t4(cond: bool, s: S, other: &mut S) {
        let x;
        if (cond) x = &mut s else x = other;
        let y = &mut s;
        *x;
        *y;
    }
}
