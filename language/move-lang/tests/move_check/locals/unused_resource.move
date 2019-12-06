module M {
    resource struct R{}

    t0() {
        let r = R{};
    }

    t1(cond: bool) {
        let r;
        if (cond) { r = R{}; };
    }

    t2(cond: bool) {
        let r;
        if (cond) {} else { r = R{}; };
    }

    t3(cond: bool) {
        let r;
        while (cond) { r = R{} };
    }

    t4(cond: bool) {
        loop { let r = R{}; }
    }

    t5() {
        let x = &R{};
    }

    t6<T>(x: T) {
    }

}
