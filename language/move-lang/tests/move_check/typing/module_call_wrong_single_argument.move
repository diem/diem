address 0x1:

module X {
    struct S {}

    public foo(s: S) {
    }

    public bar(x: u64) {
    }
}

module M {
    use 0x1::X;
    struct S {}

    public foo(s: S) {
    }

    public bar(x: u64) {
    }

    t0() {
        foo(0);
        bar(S{});
        bar(0x0);
    }

    t1() {
        X::foo(S{});
        X::foo(0);
        X::bar(S{});
        X::bar(false);
    }

}
