module 0x8675309::M {
    struct S {f: u64, b: bool}

    fun t0() {
        let x = 0;

        { let x = false; };
        (x: bool);

        { let x = false; (x: u64); };
        (x: bool);

        { let x = false; { let x = @0x0; (x: u64); }; (x: address); };
        (x: bool);
    }

    fun t1(cond: bool) {
        let x = 0;
        if (cond) {
            let (a, x) = (false, false);
            (x: u64);
            (a && x: bool);
        } else {
            let x = @0x0;
            (x: u64);
        };
        (x: address);
    }

    fun t2() {
        let x = 0;
        loop {
            let S { f: _, b: x } = S { f: 0, b: false};
            (x: u64);
            break
        };
        (x: bool);
    }
}
