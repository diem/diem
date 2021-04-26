// check that invalid abort_code values cannot be assigned
module 0x1::A {
    #[test_only]
    struct Foo has drop {}

    #[test]
    #[expected_failure(abort_code=true)]
    fun assign_boolean_abort_code() { }

    #[test]
    #[expected_failure(abort_code=x"")]
    fun assign_hexbyte_abort_code() { }

    #[test]
    #[expected_failure(abort_code=b"")]
    fun assign_byte_abort_code() { }

    #[test]
    #[expected_failure(abort_code=Foo)]
    fun assign_struct_abort_code() { }

    #[test]
    #[expected_failure(abort_code=@0xC0FFEE)]
    fun assign_address_abort_code() { }
}
