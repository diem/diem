module M {
    struct Outer { s1: Inner, s2: Inner }
    struct Inner { f1: u64, f2: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0(outer: &mut Outer) {
        let inner = &mut outer.s1;
        let c = &mut inner.f1;
        let f1 = &inner.f1;
        f1;
        c;
        inner;

        let inner = &mut outer.s1;
        let c = id_mut(&mut inner.f1);
        let f1 = &inner.f1;
        f1;
        c;
        inner;
    }

}
