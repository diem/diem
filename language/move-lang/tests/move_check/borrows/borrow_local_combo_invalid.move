module 0x8675309::M {
    struct S has copy, drop { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(cond: bool, s: S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        let x = &s;
        *f;
        *x;
    }

    fun t1(cond: bool, s: S, other: &mut S) {
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        let x = &mut s;
        *f;
        *x;
        *f;
    }

    fun t2(cond: bool, s: S) {
        let f;
        if (cond) f = &mut s.f else f = &mut s.g;
        let x = &s;
        *f;
        *x;
    }

    fun t3(cond: bool, s: S, other: &mut S) {
        let x;
        if (cond) x = &mut s else x = other;
        let y = &s;
        *y;
        *x;
        *y;
    }

    fun t4(cond: bool, s: S, other: &mut S) {
        let x;
        if (cond) x = &mut s else x = other;
        let y = &mut s;
        *y;
        *x;
    }

}
