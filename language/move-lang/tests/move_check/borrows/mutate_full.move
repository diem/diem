module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0() {
        let x = &mut 0;
        let f = x;
        *f;
        *x = 0;


        let x = &mut 0;
        let f = freeze(x);
        *f;
        *x = 0;

        let x = &mut 0;
        let f = id(x);
        *f;
        *x = 0;

        let x = &mut 0;
        let f = id_mut(x);
        *f;
        *x = 0;
    }

}
