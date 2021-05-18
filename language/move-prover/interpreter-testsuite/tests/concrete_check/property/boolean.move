module 0x2::A {
    #[test]
    public fun check_boolean_ok() {
        spec {
            assert true && false == false;
            assert true || false == true;
        };
    }

    #[test]
    public fun check_boolean_fail() {
        spec {
            assert true ==> false;
        };
    }
}
