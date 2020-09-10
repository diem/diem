module M {
    fun fn<T,>() { } // Test a trailing comma in the type parameters
    fun caller() {
        fn<u64,>()
    }
}
