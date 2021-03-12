module 0x8675309::M {
    struct S has drop { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0() {
        let s = S { f: 0, g: 0 };
        let f = &s.f;
        *f;
        s = S { f: 0, g: 0 };
        s;

        let s = S { f: 0, g: 0 };
        let f = &mut s.f;
        *f;
        s = S { f: 0, g: 0 };
        s;

        let s = S { f: 0, g: 0 };
        let f = id(&s.f);
        *f;
        s = S { f: 0, g: 0 };
        s;

        let s = S { f: 0, g: 0 };
        let f = id_mut(&mut s.f);
        *f;
        s = S { f: 0, g: 0 };
        s;
    }

    fun t1(s: &mut S) {
        let f = &s.f;
        s = &mut S { f: 0, g: 0 };
        *f;

        let f = &mut s.f;
        s = &mut S { f: 0, g: 0 };
        *f;

        let f = id(&s.f);
        s = &mut S { f: 0, g: 0 };
        *f;

        let f = id_mut(&mut s.f);
        s = &mut S { f: 0, g: 0 };
        *f;

        s;
    }

}
