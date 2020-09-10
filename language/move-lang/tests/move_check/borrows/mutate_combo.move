module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(cond: bool, _other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = &s.f else f = &s.g;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    fun t1(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    fun t2(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = s else f = other;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    fun t3(cond: bool, other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f;
        if (cond) f = id_mut(s) else f = other;
        *f;
        *s = S { f: 0, g: 0 };
        s;
    }

    fun t4(cond: bool, _other: &mut S) {
        let s = &mut S { f: 0, g: 0 };
        let f = &s.f;
        *f;
        if (cond) *s = S { f: 0, g: 0 };
        s;
    }

}
