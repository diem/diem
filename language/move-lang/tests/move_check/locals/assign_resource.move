module M {
    resource struct R{}

    t0() {
        let r = R{};
        r = R{};
        R{} = r;
    }

    t1(cond: bool) {
        let r = R{};
        if (cond) { r = R{}; };
        R{} = r;
    }

    t2(cond: bool) {
        let r = R{};
        if (cond) {} else { r = R{}; };
        R{} = r;
    }

    t3(cond: bool) {
        let r = R{};
        while (cond) { r = R{} };
        R{} = r;
    }

    t4(cond: bool) {
        let r = R{};
        loop { r = R{}; R {} = r }
    }

    t5<T>(x: T, y: T): T {
        x = y;
        x
    }
}
