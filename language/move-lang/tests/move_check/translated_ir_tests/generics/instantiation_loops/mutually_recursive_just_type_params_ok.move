module M {
    fun f<T>() {
        g<T>()
    }

    fun g<T>() {
        f<T>()
    }
}
