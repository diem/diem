module 0x8675309::M {
    struct Outer has copy, drop { s1: Inner, s2: Inner }
    struct Inner has copy, drop { f1: u64, f2: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(outer: &mut Outer) {
        let inner = &outer.s1;
        let c = &inner.f1;
        let f1 = &inner.f1;
        *c;
        *inner;
        *f1;
        *inner;
        *c;

        let inner = &outer.s1;
        let c = id(&inner.f1);
        let f1 = &inner.f1;
        *c;
        *inner;
        *f1;
        *inner;
        *c;

        let inner = &mut outer.s1;
        let c = &inner.f1;
        let f1 = &inner.f1;
        *c;
        *inner;
        *f1;
        *inner;
        *c;

        let inner = &mut outer.s1;
        let c = id(&inner.f1);
        let f1 = &inner.f1;
        *c;
        *inner;
        *f1;
        *inner;
        *c;

        let inner = &mut outer.s1;
        let c = &mut inner.f1;
        let f1 = &mut inner.f1;
        *c;
        *f1;
        *inner;

        let inner = &mut outer.s1;
        let c = id_mut(&mut inner.f1);
        let f1 = &mut inner.f1;
        *c;
        *f1;
        *inner;
    }

}
