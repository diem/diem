module 0x8675309::M {
    struct S<T> { f: T }

    fun f<T1, T2, T3>() {
        g<T2, T3, T1>()
    }

    fun g<T1, T2, T3>() {
        h<T1, T2, S<T3>>()
    }

    fun h<T1, T2, T3>() {
        // The bool breaks the chain.
        f<T1, bool, T3>()
    }
}
