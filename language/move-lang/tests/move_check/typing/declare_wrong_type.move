module M {
    resource struct R {f: u64}
    struct S { g: u64 }

    t0() {
        let S { g } : R;
        let (S { g }, R { f }): (R, R);
        g = 0;
        f = 0;
    }
}
