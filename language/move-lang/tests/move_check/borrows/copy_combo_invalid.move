module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t1(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s.f else f = &mut other.f;
        copy s;
        *f;
        s;
    }

    fun t2(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = &mut s else f = other;
        copy s;
        *f;
        s;
    }

    fun t3(cond: bool, other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f;
        if (cond) f = id_mut(&mut s) else f = other;
        copy s;
        *f;
        s;
    }

    fun t4(cond: bool, _other: &mut S) {
        let s = S { f: 0, g: 0 };
        let f = &mut s.f;
        if (cond) { copy s; };
        *f;
        s;
    }

}
