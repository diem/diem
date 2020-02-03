module M {
    t0() {
        while (true) ();
        while (false) ()
    }

    t1() {
        while ({ let foo = true; foo }) ();
        while ({ let bar = false; bar }) ()
    }

}
