module 0x8675309::M {
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

    fun t1() {
        let v = 0;

        let x = &mut v;
        let y = &v;
        x;
        y;
    }

}
