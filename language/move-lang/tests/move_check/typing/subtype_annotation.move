module M {
    struct S {}

    fun t0() {
        (&mut 0: &mut u64);
        (&mut 0: &u64);
        (&0: &u64);

        (&mut S{}: &mut S);
        (&mut S{}: &S);
        (&S{}: &S);
    }

    fun t1() {
        ((&mut 0, &mut 0): (&mut u64, &mut u64));
        ((&mut 0, &mut 0): (&mut u64, &u64));
        ((&mut 0, &mut 0): (&u64, &mut u64));
        ((&mut 0, &mut 0): (&u64, &u64));
    }

}
