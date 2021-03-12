module 0x8675309::M {
    struct Outer { s1: Inner, s2: Inner }
    struct Inner { f1: u64, f2: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(cond: bool, outer: &mut Outer, _other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = copy inner else c = &mut outer.s1;
        let f1 = &inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    fun t1(cond: bool, outer: &mut Outer, _other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = id_mut(copy inner) else c = &mut outer.s1;
        let f1 = &inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    fun t2(cond: bool, outer: &mut Outer, _other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = copy inner else c = &mut outer.s1;
        let f1 = &mut inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    fun t3(cond: bool, outer: &mut Outer, _other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = id_mut(copy inner) else c = &mut outer.s1;
        let f1 = &mut inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    fun t4(cond: bool, outer: &mut Outer, _other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = &mut inner.f1 else c = &mut inner.f2;
        let f1 = &inner.f1;
        f1;
        c;
        inner;
    }

    fun t5(cond: bool, outer: &mut Outer, _other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = id_mut(&mut inner.f1) else c = &mut inner.f1;
        let f1 = &inner.f1;
        f1;
        c;
        inner;
    }

}
