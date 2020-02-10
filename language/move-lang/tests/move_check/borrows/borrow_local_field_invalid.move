module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0() {
        let v = S { f: 0, g: 0 };
        let f = &v.f;
        let s = &mut v;
        *s = S { f: 0, g: 0 };
        f;

        let v = S { f: 0, g: 0 };
        let f = &mut v.f;
        let s = &v;
        s;
        f;
    }
}
