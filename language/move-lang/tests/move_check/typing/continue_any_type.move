module 0x8675309::M {
    struct Coin {}

    fun t0() {
        while (true) {
            0 + continue;
        }
    }

    fun t1() {
        while (true) {
            foo(continue)
        }
    }


    fun foo(c: Coin) {
        Coin {} = c;
    }
}
