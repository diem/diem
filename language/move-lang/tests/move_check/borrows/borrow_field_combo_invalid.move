module M {
    struct Outer { s1: Inner, s2: Inner }
    struct Inner { f1: u64, f2: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(cond: bool, outer: &mut Outer, other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = copy inner else c = &mut outer.s1;
        let f1 = &inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    t1(cond: bool, outer: &mut Outer, other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = id_mut(copy inner) else c = &mut outer.s1;
        let f1 = &inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    t2(cond: bool, outer: &mut Outer, other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = copy inner else c = &mut outer.s1;
        let f1 = &mut inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    t3(cond: bool, outer: &mut Outer, other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = id_mut(copy inner) else c = &mut outer.s1;
        let f1 = &mut inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

    t4(cond: bool, outer: &mut Outer, other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = &mut inner.f1 else c = &mut inner.f2;
        let f1 = &inner.f1;
        f1;
        c;
        inner;
    }

    t5(cond: bool, outer: &mut Outer, other: &mut Outer) {
        let inner = &mut outer.s1;
        let c; if (cond) c = id_mut(&mut inner.f1) else c = &mut inner.f1;
        let f1 = &inner.f1;
        f1;
        c;
        inner;
    }

}
