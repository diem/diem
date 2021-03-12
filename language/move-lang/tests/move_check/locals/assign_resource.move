module 0x8675309::M {
    struct R {}

    fun t0() {
        let r = R{};
        r = R{};
        R{} = r;
    }

    fun t1(cond: bool) {
        let r = R{};
        if (cond) { r = R{}; };
        R{} = r;
    }

    fun t2(cond: bool) {
        let r = R{};
        if (cond) {} else { r = R{}; };
        R{} = r;
    }

    fun t3(cond: bool) {
        let r = R{};
        while (cond) { r = R{} };
        R{} = r;
    }

    fun t4() {
        let r = R{};
        loop { r = R{}; R {} = r }
    }

    fun t5<T>(x: T, y: T): T {
        x = y;
        x
    }
}
