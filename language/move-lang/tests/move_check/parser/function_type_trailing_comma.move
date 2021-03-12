module 0x8675309::M {
    fun fn<T,>() { } // Test a trailing comma in the type parameters
    fun caller() {
        fn<u64,>()
    }
}
