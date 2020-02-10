module M {
    fun t0() {
        while (()) ();
        while ((())) ();
        while ({}) ()
    }

    fun t1<T: copyable>(x: T) {
        while (x) ();
        while (0) ();
        while (0x0) ()
    }

    fun t2() {
        while ((false, true)) ();
        while ((0, false)) ()
    }

}
