module 0x8675309::M {
    struct R {
        f: bool
    }

    fun t0(x: bool, r: R) {
        true && false;
        false && true;
        true && false;
        (true) && (true);
        copy x && move x;
        r.f && r.f;
        true && false && (true && false);
        let R {f: _} = r;
    }
}
