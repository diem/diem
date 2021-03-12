module 0x8675309::M {
    struct S {}

    fun t0() {
        (&0: &mut u64);
    }

    fun t1() {
        ((&0, &0): (&mut u64, &mut u64));
        ((&0, &0): (&mut u64, &u64));
        ((&0, &0): (&u64, &mut u64));
    }

}
