address 0x1:

module X {
    public foo(): u64 { 0 }
    public bar(x: u64): (address, u64) {
        (0x0, x)
    }
    public baz<T1, T2>(a: T1, x: T2): (bool, T1, T2) {
        (false, a, x)
    }
    public bing(b: bool, a: address, x: u64) {

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
    bing(b: bool, a: address, x: u64) {
    }

    t0() {
        (X::foo(): u64);
        (X::bar(0): (address, u64));
        (X::baz(0x0, 0): (bool, address, u64));
        (X::bing(false, 0x0, 0): ());
    }

    t1() {
        (foo(): u64);
        (bar(0): (address, u64));
        (baz(0x0, 0): (bool, address, u64));
        (bing(false, 0x0, 0): ());
    }

    t2() {
        let () = X::bing(X::baz(X::bar(X::foo())));
        let () = X::bing (X::baz (X::bar (X::foo())));
        let () = X::bing (X::baz (X::bar(1)));
        let () = X::bing (X::baz (0x0, 1));
        let () = X::bing (false, 0x0, 1);
    }

    t3() {
        let () = bing(baz(bar(foo())));
        let () = bing (baz (bar (foo())));
        let () = bing (baz (bar(1)));
        let () = bing (baz (0x0, 1));
        let () = bing (false, 0x0, 1);
    }

}
