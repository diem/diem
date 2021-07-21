module 0x8675309::M {
    struct S<T> { f: T }

    fun f<T>() {
        g<S<T>>()
    }

    fun g<T>() {
        f<u64>()
    }
}
