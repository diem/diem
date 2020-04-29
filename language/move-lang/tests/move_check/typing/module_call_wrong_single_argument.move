address 0x1 {

module X {
    struct S {}

    public fun foo(s: S) {
    }

    public fun bar(x: u64) {
    }
}

module M {
    use 0x1::X;
    struct S {}

    public fun foo(s: S) {
    }

    public fun bar(x: u64) {
    }

    fun t0() {
        foo(0);
        bar(S{});
        bar(0x0);
    }

    fun t1() {
        X::foo(S{});
        X::foo(0);
        X::bar(S{});
        X::bar(false);
    }

}

}
