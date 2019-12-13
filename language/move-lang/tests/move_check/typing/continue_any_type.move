module M {
    resource struct Coin {}

    t0() {
        while (true) {
            0 + continue;
        }
    }

    t1() {
        while (true) {
            foo(continue)
        }
    }


    foo(c: Coin) {
        Coin {} = c;
    }
}
