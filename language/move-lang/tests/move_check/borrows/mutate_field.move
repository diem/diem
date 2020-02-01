module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }
    t1(s: &mut S) {
        let f = &s.f;
        *f;
        s.g = 0;
        *s = S { f: 0, g: 0 };

        let f = &mut s.f;
        *f;
        s.g = 0;
        *s = S { f: 0, g: 0 };

        let f = id(&s.f);
        *f;
        s.g = 0;
        *s = S { f: 0, g: 0 };

        let f = id_mut(&mut s.f);
        *f;
        s.g = 0;
        *s = S { f: 0, g: 0 };
    }

}
