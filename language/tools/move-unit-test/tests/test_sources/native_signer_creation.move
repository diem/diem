module 0x1::M {
    use Std::UnitTest;
    use Std::Vector;
    use Std::Signer;

    struct A has key {}

    fun has_a(a: address): bool {
        exists<A>(a)
    }

    #[test_only]
    fun setup_storage(sig: &signer) {
        move_to(sig, A { })
    }

    #[test]
    fun test_exists() {
        let num_signers = 10;
        let i = 0;

        let signers = UnitTest::create_signers_for_testing(num_signers);
        while (i < num_signers) {
            setup_storage(Vector::borrow(&signers, i));
            i = i + 1;
        };

        i = 0;
        while (i < num_signers) {
            assert(has_a(Signer::address_of(Vector::borrow(&signers, i))), 0);
            i = i + 1;
        }
    }

    #[test]
    fun test_doesnt_exist() {
        let num_signers = 10;
        let i = 0;

        let signers = UnitTest::create_signers_for_testing(num_signers);
        while (i < num_signers) {
            setup_storage(Vector::borrow(&signers, i));
            i = i + 1;
        };

        // abort to trigger a dump of storage state to make sure this is getting populated correctly
        abort 0

    }

    #[test]
    fun test_determinisim() {
        let num_signers = 10;
        let i = 0;
        let signers = UnitTest::create_signers_for_testing(num_signers);
        let other_signers = UnitTest::create_signers_for_testing(num_signers);

        while (i < num_signers) {
            assert(
                Signer::address_of(Vector::borrow(&signers, i)) ==
                  Signer::address_of(Vector::borrow(&other_signers, i)),
                i
            );
            i = i + 1;
        };
    }
}
