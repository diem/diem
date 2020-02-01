module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0() {
        let x = 0;
        let f = &x;
        x;
        *f;
        x;

        let x = 0;
        let f = &mut x;
        x;
        *f;
        x;

        let x = 0;
        let f = id(&x);
        x;
        *f;

        let x = 0;
        let f = id_mut(&mut x);
        x;
        *f;
        x;
    }

}
