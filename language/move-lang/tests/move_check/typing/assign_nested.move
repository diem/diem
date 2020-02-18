module A {

    struct S { f: u64 }

    fun four(): (u64, u64, u64, u64) {
        (0, 1, 2, 3)
    }

    public fun mixed() {
        let x;
        let r = 0;
        let r_ref = &mut r;
        let s = S { f: 0 };
        (_, x, _, s.f) = four();
    }

}
