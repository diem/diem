module 0x8675309::M {
    struct S {}

    fun t0() {
        let x: &u64 = &mut 0; x;
    }

    fun t1() {
        let (x, y): (&mut u64, &u64) = (&mut 0, &mut 0); x; y;
        let (x, y): (&u64, &mut u64) = (&mut 0, &mut 0); x; y;
        let (x, y): (&u64, &u64) = (&mut 0, &mut 0); x; y;
    }

}
