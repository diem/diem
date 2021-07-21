module 0x8675309::M {
    struct R {}
    struct B<T> { f: T }
    fun fn<T>() { }
    fun caller() {
        fn<B<R>>(); // make sure '>>' is not parsed as a shift operator
        fn<B<R,>>(); // also test with trailing comma
    }
}
