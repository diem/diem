module M {
    struct X1 { x2: X2 }
    struct X2 { x3: X3 }
    struct X3 { f: u64, }

    fun t0(x1: &X1, x1_mut: &mut X1) {
        &x1.f;
        &x1.x2.f;
        &x1.x2.x3.g;

        &x1_mut.f;
        &x1_mut.x2.f;
        &x1_mut.x2.x3.g;

        &mut x1_mut.f;
        &mut x1_mut.x2.f;
        &mut x1_mut.x2.x3.g;
    }
}
