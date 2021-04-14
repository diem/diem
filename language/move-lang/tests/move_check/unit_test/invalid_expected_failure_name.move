// straight assignment of an abort code to the expected_failure attribute isn't allowed
module 0x1::M {
    #[test]
    #[expected_failure=1]
    fun f() { }
}
