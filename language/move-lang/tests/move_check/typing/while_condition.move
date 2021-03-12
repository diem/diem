module 0x8675309::M {
    fun t0() {
        while (true) ();
        while (false) ()
    }

    fun t1() {
        while ({ let foo = true; foo }) ();
        while ({ let bar = false; bar }) ()
    }

}
