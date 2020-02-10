module M {
    struct X1 { x2: X2 }
    struct X2 { x3: X3 }
    struct X3 { f: u64, }

    fun t0(x1: &X1, x1_mut: &mut X1) {
        (&x1.x2: &X2);
        (&x1.x2.x3: &X3);
        (&x1.x2.x3.f: &u64);

        (&x1_mut.x2: &X2);
        (&x1_mut.x2.x3: &X3);
        (&x1_mut.x2.x3.f: &u64);

        (&mut x1_mut.x2: &mut X2);
        (&mut x1_mut.x2.x3: &mut X3);
        (&mut x1_mut.x2.x3.f: &mut u64);
    }
}
