module M {
    fn<T,>() { } // Test a trailing comma in the type parameters
    caller() {
        fn<u64,>()
    }
}
