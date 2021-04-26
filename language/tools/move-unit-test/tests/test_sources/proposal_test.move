// This is a test based on the example in the unit testing proposal
address 0x1 {
module TestonlyModule {
    #[test_only]
    public fun aborts() {
        abort 42
    }
}

module Module {
    struct B<T> has key { t: u64 }
    struct A has key {t: u64}

    fun a(a: u64): bool {
        a == 10
    }

    fun b(a: address): bool {
        exists<A>(a)
    }

    fun c(a: address, x: u64): bool
    acquires A {
        exists<A>(a) && borrow_global<A>(a).t == x
    }

    fun d<T: store>(a: address, x: u64): bool
    acquires B {
        exists<B<T>>(a) && borrow_global<B<T>>(a).t == x
    }

    fun aborts() {
       abort 10
    }

    ///////////////////////////////////////////////////////////////////////////
    // unit tests
    ///////////////////////////////////////////////////////////////////////////

    // A test-only module import
    #[test_only]
    use 0x1::TestonlyModule;

    // A test only struct. This will only be included in test mode.
    #[test_only]
    struct C<T> has drop, key, store { x: T }

    // Not a test entrypoint, can only be called from #[test] and #[test_only] functions
    #[test_only]
    fun setup_storage_tests_b(a1: &signer, a2: &signer) {
        move_to(a1, A{t : 5});
        move_to(a2, A{t : 5});
    }

    #[test_only]
    fun setup_storage_tests_c(a1: &signer, a2: &signer) {
        move_to(a1, A{t : 5});
        move_to(a2, A{t : 6});
    }

    #[test_only]
    fun setup_storage_tests_d(a1: &signer, a2: &signer) {
        move_to(a1, B<u64>{t : 5});
        move_to(a2, B<u64>{t : 5});
        move_to(a2, B<bool>{t : 6});
        move_to(a2, B<C<u64>> {t: 5 });
    }

    #[test] // test entry point.
    fun tests_a() { // an actual test that will be run
        assert(a(0) == false, 0);
        assert(a(10), 1);
    }

    #[test(a1=@0x1, a2=@0x2)] // testing framework will create and pass-in signers with these addresses
    fun tests_b(a1: signer, a2: signer) {
        assert(b(@0x1) == false, 0);
        setup_storage_tests_b(&a1, &a2);
        assert(b(@0x1), 1);
        assert(b(@0x2), 2);
        assert(!b(@0x3), 3);
    }

    #[test(a1=@0x1, a2=@0x2)]
    fun tests_c(a1: signer, a2: signer)
    acquires A {
        setup_storage_tests_c(&a1, &a2);
        assert(c(@0x1, 5), 0);
        assert(!c(@0x1, 6), 1);
        assert(!c(@0x2, 5), 2);
        assert(c(@0x2, 6), 3);
    }

    #[test(a1=@0x1, a2=@0x2)]
    fun tests_d(a1: signer, a2: signer)
    acquires B {
        setup_storage_tests_d(&a1, &a2);
        assert(d<u64>(@0x1, 5), 0);
        assert(!d<u64>(@0x1, 6), 1);
        assert(!d<bool>(@0x2, 5), 2);
        // This should fail, and the error should be reported against the source
        assert(d<u64>(@0x2, 6), 3);
    }

    #[test]
    #[expected_failure(abort_code=10)] // check that this test aborts with the expected error code
    fun tests_aborts() {
       aborts()
    }

    #[test]
    #[expected_failure(abort_code=42)]
    fun other_module_aborts() {
       TestonlyModule::aborts()
    }
}
}
