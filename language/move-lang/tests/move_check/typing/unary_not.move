module 0x8675309::M {
    struct R {
        f: bool
    }

    fun t0(x: bool, r: R) {
        !true;
        !false;
        !x;
        !copy x;
        !move x;
        !r.f;
        let R {f: _} = r;
    }
}
