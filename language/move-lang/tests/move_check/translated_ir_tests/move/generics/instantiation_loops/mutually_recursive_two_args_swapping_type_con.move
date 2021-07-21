module 0x8675309::M {
    struct S<T> { f: T }

    fun f<T1, T2, T3>() {
        g<T2, T1>()
    }

    fun g<T1, T2>() {
        f<T1, S<T2>, u64>()
    }
}
