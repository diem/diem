address 0x1 {
module M {
    #[test, expected_failure]
    fun fail() { }

    #[test, expected_failure(abort_code=0)]
    fun fail_with_code() { }
}
}
