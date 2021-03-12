module 0x8675309::M {
    fun f<T1, T2, T3>() {
        g<T2, T3, T1>()
    }

    fun g<T1, T2, T3>() {
        h<T1, T2, T3>()
    }

    fun h<T1, T2, T3>() {
        f<T1, T2, T3>()
    }
}
