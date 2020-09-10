module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(s: &mut S) {
        let f = &mut s.f;
        *f;
        *s;
    }

    fun t1(s: &mut S) {
        let f = &s.f;
        *s;
        *f;
        *s;
    }

}
