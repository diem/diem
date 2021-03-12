module 0x8675309::M {
    struct S<T> { b: bool }

    fun f<T>() {
        g<S<T>>()
    }

    fun g<T>() {
        f<u64>()
    }
}
