module M {
    resource struct Coin {}

    fun t0() {
        while (true) {
            0 + break;
        }
    }

    fun t1() {
        while (true) {
            foo(break)
        }
    }


    fun foo(c: Coin) {
        Coin {} = c;
    }
}
