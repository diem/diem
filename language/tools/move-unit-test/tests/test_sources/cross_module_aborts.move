address 0x1 {
module M {
    #[test_only]
    public fun this_aborts() {
        abort 0
    }

    #[test]
    fun dummy_test() { }
}

module B {

    #[test_only]
    use 0x1::M;

    #[test]
    fun failing_test() {
        M::this_aborts()
    }
}
}
