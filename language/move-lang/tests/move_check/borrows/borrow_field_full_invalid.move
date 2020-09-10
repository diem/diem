module M {
    struct Outer { s1: Inner, s2: Inner }
    struct Inner { f1: u64, f2: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(outer: &mut Outer) {
        let inner = &mut outer.s1;
        let c = copy inner;
        let f1 = &inner.f1;
        c;
        inner;
        f1;
        inner;
        c;

        let inner = &mut outer.s1;
        let c = id_mut(copy inner);
        let f1 = &inner.f1;
        c;
        inner;
        f1;
        inner;
        c;

        let inner = &mut outer.s1;
        let c = copy inner;
        let f1 = &mut inner.f1;
        c;
        inner;
        f1;
        inner;
        c;

        let inner = &mut outer.s1;
        let c = id_mut(copy inner);
        let f1 = &mut inner.f1;
        c;
        inner;
        f1;
        inner;
        c;
    }

}
