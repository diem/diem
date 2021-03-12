module 0x8675309::M {
    struct R {f: u64}

    fun t0() {
        let (): ();
        let x: u64;
        let (x, b, R{f}): (u64, bool, R);
    }
}
