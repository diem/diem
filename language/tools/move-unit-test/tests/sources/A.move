module 0x1::A {
    #[test]
    fun a() { }

    #[test_only]
    public fun a_call() {
        abort 0
    }
}
