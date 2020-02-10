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
        *s;
        *f;

        let f = id_mut(&mut s.f);
        *s;
        *f;
    }
}
