module M {
    resource struct R{}

    fun t0() {
        let r = R{};
    }

    fun t1(cond: bool) {
        let r;
        if (cond) { r = R{}; };
    }

    fun t2(cond: bool) {
        let r;
        if (cond) {} else { r = R{}; };
    }

    fun t3(cond: bool) {
        let r;
        while (cond) { r = R{} };
    }

    fun t4() {
        loop { let r = R{}; }
    }

    fun t5() {
        let _ = &R{};
    }

    fun t6<T>(_x: R) {
    }

}
