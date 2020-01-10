module M {
    foo(): u64 { 0 }
    bar(x: u64): (address, u64) {
        (0x0, x)
    }
    baz<T1, T2>(a: T1, x: T2): (bool, T1, T2) {
        (false, a, x)
    }

    t0(cond: bool) {
        foo (if (cond) () else ());
        bar (if (cond) 0 else 0);
        baz (if (cond) (false, 0x0) else (true, 0x1));
    }

    t1(cond: bool) {
        foo(if (cond) () else ());
        bar(if (cond) 0 else 0);
        baz(if (cond) (false, 0x0) else (true, 0x1));
    }

    t2() {
        foo({});
        foo({ let _x = 0; });

        let x = 0;
        bar({ x });
        bar({ let x = 0; x });

        let a = 0x0;
        baz({ (a, x) });
        baz({ let a = false; (a, x) });
    }

    t3() {
        foo({});
        foo({ let _x = 0; });

        let x = 0;
        bar({ x });
        bar({ let x = 0; x });

        let a = 0x0;
        baz({ (a, x) });
        baz({ let a = false; (a, x) });
    }
}
