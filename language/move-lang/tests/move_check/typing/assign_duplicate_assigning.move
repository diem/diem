module M {
    resource struct R {f: u64}

    fun t0() {
        let x;
        (x, x) = (0, 0);
        let f;
        (f, R{f}, f) = (0, R { f: 0 }, 0);
    }
}
