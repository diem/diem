module M {
    resource struct R {f: u64}

    t0() {
        let (): () = ();
        let x: u64 = 0;
        let (x, b, R{f}): (u64, bool, R) = (0, false, R { f: 0 });
    }
}
