module 0x8675309::M {
    struct S has copy, drop { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0() {
        let v = S { f: 0, g: 0 };
        let f = &v.f;
        let s = &v;
        *f;
        *s;
        *f;

        let v = S { f: 0, g: 0 };
        let f = id(&v.f);
        let s = &v;
        *f;
        *s;
        *f;

        let v = S { f: 0, g: 0 };
        let f = &v.f;
        let s = &mut v;
        *f;
        *s;
        *f;
    }

}
