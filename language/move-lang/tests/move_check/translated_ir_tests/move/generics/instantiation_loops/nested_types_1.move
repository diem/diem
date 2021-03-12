module 0x8675309::M {
    struct S<T> { b: bool }

    fun foo<T>() {
        foo<S<S<T>>>()
    }
}
