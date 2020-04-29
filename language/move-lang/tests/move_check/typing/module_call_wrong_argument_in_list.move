address 0x1 {

module X {
    struct S {}
    public fun s(): S {
        S{}
    }
    public fun foo(a: address, u: u64, s: S) {
    }
}

module M {
    use 0x1::X;
    struct S {}

    public fun foo(a: address, u: u64, s: S) {
    }

    fun t0() {
        foo(false, 0, S{});
        foo(0x0, false, S{});
        foo(0x0, 0, false);
        foo(0x0, false, false);
        foo(false, 0, false);
        foo(false, false, S{});
    }

    fun t1() {
        X::foo(false, 0, X::s());
        X::foo(0x0, false, X::s());
        X::foo(0x0, 0, S{});
        X::foo(0x0, false, S{});
        X::foo(false, 0, S{});
        X::foo(false, false, X::s());
    }

}

}
