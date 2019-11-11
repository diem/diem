module M {
    t0() {
        while (true) ();
        while (false) ()
    }

    t1() {
        while ({ let x = true; x }) ();
        while ({ let x = false; x }) ()
    }

}
