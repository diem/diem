module M {
    resource struct R{}

    fun t1(cond: bool) {
        let r: R;
        if (cond) { r = R{}; };
        r = R{};
        R{} = r;
    }

    fun t2(cond: bool) {
        let r: R;
        if (cond) {} else { r = R{}; };
        r = R{};
        R{} = r;
    }

    fun t3(cond: bool) {
        let r: R;
        while (cond) { r = R{} };
        r = R{};
        R{} = r;
    }

    fun t4(cond: bool) {
        let r: R;
        loop { r = R{} }
    }

    fun t5<T>(cond: bool, x: T, y: T): (T, T) {
        if (cond) { x = y };
        (x, y)
    }
}
