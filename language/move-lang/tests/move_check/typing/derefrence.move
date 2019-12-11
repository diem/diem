module M {
    struct X {}
    struct S { f: u64, x: X }

    t0(x: &u64, x_mut: &mut u64, s: &S, s_mut: &mut S){
        (*x : u64);
        (*x_mut: u64);

        (*s: S);
        (*&s.f: u64);
        (s.f: u64);
        (*&s.x: X);

        (*s_mut: S);
        (*&s_mut.f: u64);
        (*&mut s_mut.f: u64);
        (s_mut.f: u64);
        (*&s_mut.x: X);
        (*&mut s_mut.x: X);

    }

}
