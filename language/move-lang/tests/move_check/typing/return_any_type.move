module M {
    resource struct Coin {}

    fun t0() {
        0 + (return ());
    }

    fun t1() {
        foo(return ());
    }


    fun foo(c: Coin) {
        Coin {} = c;
    }
}
