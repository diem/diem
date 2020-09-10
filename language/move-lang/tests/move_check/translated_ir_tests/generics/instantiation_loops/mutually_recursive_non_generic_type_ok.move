module M {
    struct S<T> { b: bool }

    fun f<T>() {
        g<S<T>>()
    }

    fun g<T>() {
        f<u64>()
    }
}
