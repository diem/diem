module 0x8675309::M {
    struct S has drop {}

    fun t0() {
        ();
    }

    fun t1() {
        0;
    }

    fun t2() {
        (0, false, S{});
    }

    fun t3() {
        if (true) (0, false, S{}) else (0, false, S{});
    }
}
