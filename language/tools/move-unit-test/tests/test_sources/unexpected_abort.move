address 0x1 {
module M {
    #[test]
    public fun unexpected_abort() {
        abort 0
    }

    #[test]
    #[expected_failure(abort_code=1)]
    public fun wrong_abort_code() {
        abort 0
    }

    #[test]
    #[expected_failure(abort_code=0)]
    public fun correct_abort_code() {
        abort 0
    }

    #[test]
    #[expected_failure]
    public fun just_test_failure() {
        abort 0
    }

    #[test_only]
    fun abort_in_other_function() {
        abort 1
    }

    #[test]
    fun unexpected_abort_in_other_function() {
        abort_in_other_function()
    }
}
}
