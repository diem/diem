module M {
    resource struct R {}

    t0() {
        R{};
    }

    t1() {
        let r = R{};
        r;
    }

    t2() {
        (0, false, R{});
    }

    t3() {
        let r = R{};
        if (true) (0, false, R{}) else (0, false, r);
    }
}
