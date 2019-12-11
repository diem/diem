module M {
    resource struct R {f: u64}

    t0() {
        let x: () = ();
        let (): u64 = 0;
        let (x, b, R{f}): (u64, bool, R, R) = (0, false, R{f: 0}, R{f: 0});
        let (x, b, R{f}): (u64, bool) = (0, false);
    }
}
