module 0x8675309::M {
    struct R {}

    fun t0() {
        R{};
    }

    fun t1() {
        let r = R{};
        r;
    }

    fun t2() {
        (0, false, R{});
    }

    fun t3() {
        let r = R{};
        if (true) (0, false, R{}) else (0, false, r);
    }
}
