module 0x1::B {
    #[test_only]
    use 0x1::A;

    #[test]
    fun b() { }

    #[test]
    public fun b_other() {
        A::a_call()
    }

    #[test]
    #[expected_failure(abort_code = 0)]
    public fun b_other0() {
        A::a_call()
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    public fun b_other1() {
        A::a_call()
    }
}
