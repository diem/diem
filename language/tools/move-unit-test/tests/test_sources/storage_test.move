address 0x1 {
module B {
    struct A<T> has key { t: T }

    fun a<T: store>(a: address): bool {
        exists<A<T>>(a)
    }

    fun b(a: address, x: u64): bool
    acquires A {
        exists<A<u64>>(a) && borrow_global<A<u64>>(a).t == x
    }


    #[test_only]
    fun setup_storage_tests_a(a1: &signer, a2: &signer) {
        move_to(a1, A<u64>{t : 5});
        move_to(a2, A<bool>{t : false});
    }

    #[test_only]
    fun setup_storage_tests_b(a1: &signer, a2: &signer) {
        move_to(a1, A{t : 5});
        move_to(a2, A{t : 6});
    }

    #[test(a1=0x1, a2=0x2)] // testing framework will create and pass-in signers with these addresses
    fun tests_a(a1: signer, a2: signer) {
        assert(!a<u64>(0x1), 0);
        setup_storage_tests_a(&a1, &a2);
        assert(a<u64>(0x1), 1);
        assert(a<bool>(0x2), 2);
        assert(!a<u64>(0x3), 3);
        assert(!a<bool>(0x3), 4);
        assert(!a<address>(0x1), 5);
    }

    #[test(a1=0x1, a2=0x2)]
    fun tests_b(a1: signer, a2: signer)
    acquires A {
        setup_storage_tests_b(&a1, &a2);
        assert(b(0x1, 5), 0);
        assert(!b(0x1, 6), 1);
        assert(!b(0x2, 5), 2);
        assert(b(0x2, 6), 3);
    }
}
}
