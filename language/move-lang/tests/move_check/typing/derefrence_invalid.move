module M {
    struct X {}
    struct S { f: u64, x: X }

    fun t0(x: &u64, x_mut: &mut u64, s: &S, s_mut: &mut S){
        (*x : bool);
        (*x_mut: &u64);

        (*s: X);
        (*&s.f: bool);
        (s.f: &u64);
        (*&s.x: &X);

        (*s_mut: X);
        (*&s_mut.f: bool);
        (*&mut s_mut.f: (bool, u64));
        (s_mut.f: &u64);
        (*&s_mut.x: (X, S));
        (*&mut s_mut.x: ());

    }

}
