module 0x8675309::M {
    fun t0() {
        if (()) () else ();
        if ((())) () else ();
        if ({}) () else ()
    }

    fun t1<T: drop>(x: T) {
        if (x) () else ();
        if (0) () else ();
        if (@0x0) () else ()
    }

    fun t2() {
        if ((false, true)) () else ();
        if ((0, false)) () else ()
    }

}
