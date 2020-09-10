module M {
    resource struct R{}

    fun t0() {
        let r = R{};
        return ()
    }

    fun t1(cond: bool) {
        let r = R {};
        if (cond) { return () };
        R {} = r;
    }

    fun t2(cond: bool) {
        let r = R{};
        if (cond) {} else { return () };
        R {} = r;
    }

    fun t3(cond: bool) {
        let r = R {};
        while (cond) { return () };
        R {} = r;
    }

    fun t4() {
        let r = R{};
        loop { return () }
    }

    fun t5() {
        let x = &R{};
        return ()
    }

    fun t6<T>(_x: R) {
        return ()
    }
}
