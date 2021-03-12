module 0x8675309::M {
    struct S { f: u64 }

    fun t1(root: &mut S, cond: bool) {
        let u = 0;

        let x = if (cond) {
            &u
        } else {
            move u;
            &root.f
        };
        *x;
    }
}
