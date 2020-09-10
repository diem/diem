module M {
    struct S<T> { b: bool }
    struct R<T1, T2> { b: bool }

    fun foo<T>() {
        foo<R<u64, S<S<T>>>>()
    }
}
