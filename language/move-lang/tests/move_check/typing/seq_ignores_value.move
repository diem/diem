module M {
    struct S {}

    t0() {
        ();
    }

    t1() {
        0;
    }

    t2() {
        (0, false, S{});
    }

    t3() {
        if (true) (0, false, S{}) else (0, false, S{});
    }
}
