module M {
    struct S {}

    fun t0() {
        let x: &u64;
        x = &mut 0;
        x;
    }

    fun t1() {
        let (x, y): (&mut u64, &u64);
        (x, y) = (&mut 0, &mut 0);
        x; y;

        let (x, y): (&u64, &mut u64);
        (x, y) = (&mut 0, &mut 0);
        x; y;

        let (x, y): (&u64, &u64);
        (x, y) = (&mut 0, &mut 0);
        x; y;
    }

}
