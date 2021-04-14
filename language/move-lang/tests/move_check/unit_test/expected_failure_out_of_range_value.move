// abort_code numbers need to remain in the u64 range
address 0x1 {
module M {
    // at u64::max - 1, OK
    #[test]
    #[expected_failure(abort_code=18446744073709551614)]
    fun ok() { }

    // at u64::max, OK
    #[test]
    #[expected_failure(abort_code=18446744073709551615)]
    fun at_max_ok() { }

    // at u64::max + 1, raise an error
    #[test]
    #[expected_failure(abort_code=18446744073709551616)]
    fun past_max_should_fail() { }
}
}
