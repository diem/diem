module 0x2::A {
    use Std::Signer;

    struct R has key {
        f1: bool,
        f2: u64,
    }

    #[test(s=@0x2)]
    public fun check_global_basics_ok(s: &signer) {
        let a = Signer::address_of(s);
        spec {
            assert !exists<R>(a);
        };
        let r = R { f1: true, f2: 1 };
        move_to(s, r);
        spec {
            assert exists<R>(a);
            assert global<R>(a).f1;
            assert global<R>(a) == R { f1: true, f2: 1 };
        };
    }

    #[test(s=@0x2)]
    public fun check_global_basics_fail(s: &signer) {
        let a = Signer::address_of(s);
        spec {
            assert global<R>(a).f2 == 42;
        };
    }
}
