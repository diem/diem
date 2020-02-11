module M {
    resource struct R {}
    resource struct B<T> {}
    fun fn<T>() { }
    fun caller() {
        fn<B<R>>(); // make sure '>>' is not parsed as a shift operator
        fn<B<R,>>(); // also test with trailing comma
    }
}
