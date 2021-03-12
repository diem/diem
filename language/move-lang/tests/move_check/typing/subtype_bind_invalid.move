module 0x8675309::M {
    struct S {}

    fun t0() {
        let x: &mut u64 = &0;
    }

    fun t1() {
        let (x, y): (&mut u64, &mut u64) = (&0, &0);
        let (x, y): (&mut u64, &u64) = (&0, &0);
        let (x, y): (&u64, &mut u64) = (&0, &0);
    }

}
