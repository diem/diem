module M {
    resource struct Coin {}

    t1() {
        while (true) {
            foo(continue)
        }
    }


    foo(c: Coin) {
        Coin {} = c;
    }
}
