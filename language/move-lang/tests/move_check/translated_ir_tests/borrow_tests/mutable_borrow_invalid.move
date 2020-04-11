module M {
    struct S { f: u64, g: u64, h: u64 }


    fun t1(root: &mut S, cond: bool) {
        let x = if (cond) &mut root.f else &mut root.g;

        // INVALID
        root.f = 1;
        *x;
    }

    fun t2(root: &mut S, cond: bool) {
        let x = if (cond) &mut root.f else &mut root.g;

        // INVALID
        foo(x, &mut root.f);
    }

    fun foo(_x: &mut u64, _y: &mut u64) {
    }
}
