module 0x8675309::M {
    struct S<T> {}

    fun t() {
        S{} = S{};
    }
}
