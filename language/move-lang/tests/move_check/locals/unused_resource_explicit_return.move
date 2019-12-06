module M {
    resource struct R{}

    t0() {
        let r = R{};
        return ()
    }

    t1(cond: bool) {
        let r = R {};
        if (cond) { return () };
        R {} = r;
    }

    t2(cond: bool) {
        let r = R{};
        if (cond) {} else { return () };
        R {} = r;
    }

    t3(cond: bool) {
        let r = R {};
        while (cond) { return () };
        R {} = r;
    }

    t4(cond: bool) {
        let r = R{};
        loop { return () }
    }

    t5() {
        let x = &R{};
        return ()
    }

    t6<T>(x: T) {
        return ()
    }
}
