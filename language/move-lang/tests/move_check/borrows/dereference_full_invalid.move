module 0x8675309::M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun t0() {
        let x = &mut 0;
        let y = copy x;
        *x;
        *y;

        let x = &mut 0;
        let y = id_mut(x);
        *x;
        *y;
    }
}
