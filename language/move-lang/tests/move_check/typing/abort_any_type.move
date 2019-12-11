module M {
    resource struct Coin {}

    t0() {
        0 + (abort 0);
    }

    t1() {
        foo(abort 0);
    }


    foo(c: Coin) {
        Coin {} = c;
    }
}
