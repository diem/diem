module M {
    struct S { f: u64, g: u64 }
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
        move s;


        let s = S { f: 0, g: 0 };
        let f = &mut s.f;
        *f;
        move s;

        let s = S { f: 0, g: 0 };
        let f = id(&s.f);
        *f;
        move s;

        let s = S { f: 0, g: 0 };
        let f = id_mut(&mut s.f);
        *f;
        move s;
    }
}
