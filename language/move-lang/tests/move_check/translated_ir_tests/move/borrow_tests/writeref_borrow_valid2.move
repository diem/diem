module 0x8675309::M {
    struct S { f: u64, g: u64, h: u64 }

    fun t1(root: &mut S, cond: bool) {
        let x = if (cond) &mut root.f else &mut root.g;
        root.h = 0;
        x;
    }

    fun t2(root: &mut S, cond: bool) {
        let x = if (cond) &mut root.f else &mut root.g;

        &mut root.f;
        &mut root.g;
        x;
    }
}
