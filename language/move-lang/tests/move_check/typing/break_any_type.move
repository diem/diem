module M {
    resource struct Coin {}

    t0() {
        while (true) {
            0 + break;
        }
    }

    t1() {
        while (true) {
            foo(break)
        }
    }


    foo(c: Coin) {
        Coin {} = c;
    }
}
