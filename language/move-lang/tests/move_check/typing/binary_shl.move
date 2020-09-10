module M {
    resource struct R {
        f: u64,
        b: u8,
    }

    fun t0(x: u64, b: u8, r: R) {
        0 << 0;
        1 << 0;
        0 << 1;
        0 << (1: u8);
        (0: u8) + 1;
        (0: u128) << 1;
        (0) << (1);
        copy x << copy b;
        r.f << r.b;
        1 << r.b << r.b << 0;
        R {f: _, b: _} = r
    }
}
