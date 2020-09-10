module M {
    fun t0() {
        if (true) () else ();
        if (false) () else ()
    }

    fun t1() {
        if ({ let x = true; x }) () else ();
        if ({ let x = false; x }) () else ()
    }

}
