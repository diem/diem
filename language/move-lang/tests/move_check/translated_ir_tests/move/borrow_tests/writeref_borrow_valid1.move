module M {
    struct G { v1: u64, v2: u64 }
    struct S { g1: G, g2: G }

    fun t1(root: &mut S) {
        let v1_mut = &mut root.g1.v1;
        let v2_mut = &mut root.g1.v2;
        let g2_mut = &mut root.g2;

        *g2_mut = G { v1: 0, v2: 0 };
        *v2_mut = 0;
        *v1_mut = 1;
        v2_mut;
        g2_mut;
    }
}
