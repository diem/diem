module M {
    struct G { v: u64 }
    struct S { g: G }

    fun t1(root: &mut S) {
        let v_mut = &mut root.g.v;
        let g_mut = &mut root.g;

        // INVALID
        *g_mut = G { v: 0 };
        v_mut;
    }
}
