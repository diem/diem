module M {
    struct S {}

    t0() {
        let x: &u64 = &mut 0;
    }

    t1() {
        let (x, y): (&mut u64, &u64) = (&mut 0, &mut 0);
        let (x, y): (&u64, &mut u64) = (&mut 0, &mut 0);
        let (x, y): (&u64, &u64) = (&mut 0, &mut 0);
    }

}
