module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0() {
        let v = 0;
        let x = &mut v;
        let y = &mut v;
        *x;
        *y;

        let x = id_mut(&mut v);
        let y = &mut v;
        *x;
        *y;

        let x = &v;
        let y = &mut v;
        *y;
        *x;
        *y;

        let x = &v;
        let y = &v;
        *x;
        *y;
        *x;

        let x = id(&v);
        let y = &v;
        *x;
        *y;
        *x;
    }

}
