module 0x8675309::M {
    fun f<T>() {
        g<T>()
    }

    fun g<T>() {
        f<T>()
    }
}
