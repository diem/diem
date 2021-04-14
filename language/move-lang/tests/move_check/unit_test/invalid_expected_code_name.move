// only "abort_code" can be used as the expected_failure abort code assignment name
module 0x1::M {
    #[test]
    #[expected_failure(cod=1)]
    fun no() {}

    #[test]
    #[expected_failure(code=1)]
    fun nope() {}

    #[test]
    #[expected_failure(abort_cod=1)]
    fun noo() {}
}
