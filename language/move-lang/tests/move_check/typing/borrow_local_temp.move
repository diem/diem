module M {
    struct S {}
    resource struct R {}

    t0() {
        (&true : &bool);
        (&mut false : &mut bool);
        (&0 : &u64);
        (&mut 1 : &mut u64);
        (&S {} : &S);
        (&mut S{} : &mut S);
    }
}
