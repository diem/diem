module M {
    t0() {
        while (()) ();
        while ((())) ();
        while ({}) ()
    }

    t1<T: copyable>(x: T) {
        while (x) ();
        while (0) ();
        while (0x0) ()
    }

    t2() {
        while ((false, true)) ();
        while ((0, false)) ()
    }

}
