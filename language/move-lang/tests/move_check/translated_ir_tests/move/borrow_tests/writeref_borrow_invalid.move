module 0x8675309::M {
    struct G has copy, drop { v: u64 }
    struct S has copy, drop { g: G }

    fun t1(root: &mut S) {
        let v_mut = &mut root.g.v;
        let g_mut = &mut root.g;

        // INVALID
        *g_mut = G { v: 0 };
        v_mut;
    }
}
