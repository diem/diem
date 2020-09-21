module M {
    struct S<T> { b: bool }

    fun foo<T>() {
        foo<S<S<T>>>()
    }
}
