module M {
    t0() {
        if (true) () else ();
        if (false) () else ()
    }

    t1() {
        if ({ let x = true; x }) () else ();
        if ({ let x = false; x }) () else ()
    }

}
