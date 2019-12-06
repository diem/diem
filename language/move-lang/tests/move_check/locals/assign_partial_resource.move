module M {
    resource struct R{}

    t1(cond: bool) {
        let r: R;
        if (cond) { r = R{}; };
        r = R{};
        R{} = r;
    }

    t2(cond: bool) {
        let r: R;
        if (cond) {} else { r = R{}; };
        r = R{};
        R{} = r;
    }

    t3(cond: bool) {
        let r: R;
        while (cond) { r = R{} };
        r = R{};
        R{} = r;
    }

    t4(cond: bool) {
        let r: R;
        loop { r = R{} }
    }

    t5<T>(cond: bool, x: T, y: T): (T, T) {
        if (cond) { x = y };
        (x, y)
    }
}
