module M {
    t0() {
        if (()) () else ();
        if ((())) () else ();
        if ({}) () else ()
    }

    t1<T: copyable>(x: T) {
        if (x) () else ();
        if (0) () else ();
        if (0x0) () else ()
    }

    t2() {
        if ((false, true)) () else ();
        if ((0, false)) () else ()
    }

}
