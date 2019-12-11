module M {
    struct S {}

    t0() {
        (&0: &mut u64);
    }

    t1() {
        ((&0, &0): (&mut u64, &mut u64));
        ((&0, &0): (&mut u64, &u64));
        ((&0, &0): (&u64, &mut u64));
    }

}
