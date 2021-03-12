module 0x8675309::M {
    fun f<T1, T2>() {
        g<T2, T1>();
    }

    fun g<T1, T2>() {
        f<T1, T2>();
    }
}
