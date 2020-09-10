module M {
    struct S { u: u64 }
    resource struct R {
        f: u64
    }

    fun t0(x: u64, r: R, s: S) {
        false >= true;
        1 >= false;
        false >= 1;
        0x0 >= 0x1;
        (0: u8) >= (1: u128);
        r >= r;
        s >= s;
        0 >= 1 >= 2;
        (1 >= false) && (0x0 >= 0);
        () >= ();
        1 >= ();
        (0, 1) >= (0, 1, 2);
        (1, 2) >= (0, 1);
    }
}
