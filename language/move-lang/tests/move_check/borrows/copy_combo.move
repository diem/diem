module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(cond: bool, _other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &s.f else f = &s.g;
        copy s;
        *f;
        s;
    }

    fun t1(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        *f;
        copy s;
        s;
    }

    fun t2(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s else f = other;
        *f;
        copy s;
        s;
    }

    fun t3(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = id_mut(&mut s) else f = other;
        *f;
        copy s;
        s;
    }

    fun t4(cond: bool, _other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f = &s.f;
        if (cond) { copy s; s; } else { *f; }
    }

    fun t5(cond: bool, _other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f = &s.f;
        if (cond) { copy s; };
        *f;
    }


}
