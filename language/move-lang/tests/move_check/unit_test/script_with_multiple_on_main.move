// test-related attributes are not allowed in and/or on scripts. We should get an error
// pointing to each one of the attributes in this test. Test annotations inside of a script.
script {
    #[expected_failure]
    #[test_only, test]
    fun main() { }
}
