module M {
    resource struct R {f: u64}

    t0() {
        let x: ();
        let (): u64;
        let (x, b, R{f}): (u64, bool, R, R);
        let (x, b, R{f}): (u64, bool);
    }
}
