module M {
    struct X1 { x2: X2 }
    struct X2 { x3: X3 }
    struct X3 { f: u64, }

    t0(x1: &X1, x1_mut: &mut X1, x2: &X2, x2_mut: &mut X2) {
        (x1.x2.x3.f: u64);
        (x1_mut.x2.x3.f: u64);
        (x2.x3.f: u64);
        (x2_mut.x3.f: u64);
    }
}
