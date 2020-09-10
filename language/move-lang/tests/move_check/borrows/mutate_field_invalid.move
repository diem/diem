module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }
    fun t1(s: &mut S) {
        let f = &s.f;
        *s = S { f: 0, g: 0 };
        *f;

        let f = &mut s.f;
        *s = S { f: 0, g: 0 };
        *f;

        let f = id(&s.f);
        *s = S { f: 0, g: 0 };
        *f;

        let f = id_mut(&mut s.f);
        *s = S { f: 0, g: 0 };
        *f;
        s;
    }

}
