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
