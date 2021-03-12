module 0x8675309::M {
    struct S has copy, drop { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(cond: bool) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &s.f else f = &s.g;
        s = S { f: 0, g: 0 };
        *f;
        s;
    }

    fun t1(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        s = S { f: 0, g: 0 };
        *f;
        s;
    }

    fun t2(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s else f = other;
        s = S { f: 0, g: 0 };
        *f;
        s;
    }

    fun t3(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = id_mut(&mut s) else f = other;
        s = S { f: 0, g: 0 };
        *f;
        s;
    }

    fun t4(cond: bool) {
        let s = S { f: 0, g: 0 };
        let f = &s.f;
        if (cond) s = S { f: 0, g: 0 };
        *f;
        s;
    }

}
