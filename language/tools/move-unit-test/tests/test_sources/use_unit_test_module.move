module 0x1::M {
    use 0x1::UnitTest;

    #[test]
    fun poison_call() {
        UnitTest::create_signers_for_testing(0);
    }
}
