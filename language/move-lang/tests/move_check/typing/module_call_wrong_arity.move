
address 0x1:

module X {
    public foo(): u64 { 0 }
    public bar(x: u64): (address, u64) {
        (0x0, x)
    }
    public baz<T1, T2>(a: T1, x: T2): (bool, T1, T2) {
        (false, a, x)
    }
}


module M {
    use 0x1::X;

    foo(): u64 { 0 }
    bar(x: u64): (address, u64) {
        (0x0, x)
    }
    baz<T1, T2>(a: T1, x: T2): (bool, T1, T2) {
        (false, a, x)
    }

    t0() {
        X::foo(1);
        X::foo(1, 2);
        X::bar();
        X::bar(1, 2);
        X::baz<u64, u64>();
        X::baz<u64, u64>(1);
        X::baz(1, 2, 3);
    }

    t1() {
        foo(1);
        foo(1, 2);
        bar();
        bar(1, 2);
        baz<u64, u64>();
        baz<u64, u64>(1);
        baz(1, 2, 3);
    }

}
