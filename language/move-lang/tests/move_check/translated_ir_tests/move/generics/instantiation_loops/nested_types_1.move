module 0x8675309::M {
    struct S<T> { f: T }

    fun foo<T>() {
        foo<S<S<T>>>()
    }
}
