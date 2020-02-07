module M {
    resource struct Coin {}

    t1() {
        while (true) {
            foo(break)
        }
    }


    foo(c: Coin) {
        Coin {} = c;
    }
}
