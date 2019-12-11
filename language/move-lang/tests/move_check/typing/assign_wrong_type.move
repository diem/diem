module M {
    resource struct R {f: u64}
    struct S { g: u64 }

    t0() {
        let g;
        let f;
        S { g } = R {f :0};
        (S { g }, R { f }) = (R{ f: 0 }, R{ f: 1 });
    }

    t1() {
        let x;
        let b;
        let f;
        x = ();
        () = 0;
        (x, b, R{f}) = (0, false, R{f: 0}, R{f: 0});
        (x, b, R{f}) = (0, false);
    }

    t2() {
        let x = false;
        let b = 0;
        let f = 0x0;
        let r = S{ g: 0 };
        (x, b, R{f}, r) = (0, false, R{f: 0}, R{f: 0});
    }
}
