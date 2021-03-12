module 0x8675309::M {
    struct R {}

    fun t0() {
        let r = R{};
    }

    fun t1() {
        // prefixing an unused resource with _ does not work
        let _r = R{};
    }

    fun t2(cond: bool) {
        let r;
        if (cond) { r = R{}; };
    }

    fun t3(cond: bool) {
        let r;
        if (cond) {} else { r = R{}; };
    }

    fun t4(cond: bool) {
        let r;
        while (cond) { r = R{} };
    }

    fun t5() {
        loop { let r = R{}; }
    }

    fun t6() {
        let _ = &R{};
    }

    fun t7<T>(_x: R) {
    }

}
