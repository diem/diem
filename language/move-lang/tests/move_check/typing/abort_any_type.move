module M {
    resource struct Coin {}

    fun t0() {
        0 + (abort 0);
    }

    fun t1() {
        foo(abort 0);
    }


    fun foo(c: Coin) {
        Coin {} = c;
    }
}
