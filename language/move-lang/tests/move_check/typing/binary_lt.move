module M {
    resource struct R {
        f: u64
    }

    fun t0(x: u64, r: R) {
        0 < 0;
        1 < 0;
        0 < 1;
        0 < (1: u8);
        (0: u8) < 1;
        0 < (1: u128);
        (0: u128) < 1;
        (0) < (1);
        copy x < move x;
        r.f < r.f;
        (1 < r.f) && (r.f < 0);
        let R {f: _} = r;
    }
}
