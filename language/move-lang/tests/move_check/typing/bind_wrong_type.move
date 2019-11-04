module M {
    resource struct R {f: u64}
    struct S { g: u64 }

    t0() {
        let S { g } = R {f :0};
        let (S { g }, R { f }) = (R{ f: 0 }, R{ f: 1 });
    }

    t1() {
        let x = ();
        let () = 0;
        let (x, b, R{f}) = (0, false, R{f: 0}, R{f: 0});
        let (x, b, R{f}) = (0, false);
    }

    t2() {
        let x: () = 0;
        let (): u64 = ();
        let (x, b, R{f}): (u64, bool, R, R) = (0, false, R{f: 0});
        let (x, b, R{f}): (u64, bool) = (0, false, R{f: 0});
    }
}
