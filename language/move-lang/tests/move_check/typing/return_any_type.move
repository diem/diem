module M {
    resource struct Coin {}

    t0() {
        0 + (return ());
    }

    t1() {
        foo(return ());
    }


    foo(c: Coin) {
        Coin {} = c;
    }
}
