module M {
    struct S { u: u64 }
    resource struct R {
        f: u64
    }

    fun t0(x: u64, r: R, s: S) {
        0 && 1;
        1 && false;
        false && 1;
        0x0 && 0x1;
        (0: u8) && (1: u128);
        r && r;
        s && s;
        () && ();
        true && ();
        (true, false) && (true, false, true);
        (true, true) && (false, false);
    }
}
