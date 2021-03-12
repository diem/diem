module 0x8675309::M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0(s: &mut S) {
        let f = &mut s.f;
        freeze(s);
        *f;
    }

    fun t1(s: &mut S) {
        let f = &s.f;
        let g = &mut s.f;
        freeze(s);
        *f;
        *g;
    }
}
