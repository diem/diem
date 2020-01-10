module M {
    struct S {f: u64, b: bool}

    t0() {
        let x = 0;

        { let x = false; x; };
        (x: u64);

        { let x = false; (x: bool); };
        (x: u64);

        { let x = false; { let x = 0x0; (x: address); }; (x: bool); };
        (x: u64);
    }

    t1(cond: bool) {
        let x = 0;
        if (cond) {
            let (a, x) = (false, false);
            (a && x: bool);
        } else {
            let x = 0x0;
            (x: address);
        };
        (x: u64);
    }

    t2() {
        let x = 0;
        loop {
            let S { f: _, b: x } = S { f: 0, b: false};
            (x: bool);
            break
        };
        (x: u64);
    }

}
