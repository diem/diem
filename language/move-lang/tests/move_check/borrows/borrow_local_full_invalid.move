module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0() {
        let v = 0;
        let x = &mut v;
        let y = &mut v;
        *y;
        *x;

        let x = &mut v;
        let y = id_mut(&mut v);
        *y;
        *x;

        let x = &v;
        let y = &mut v;
        *y = 0;
        *x;

    }

    t1() {
        let v = 0;

        let x = &mut v;
        let y = &v;
        x;
        y;
    }

}
