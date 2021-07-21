module 0x8675309::M {
    struct S<phantom T> {}

    fun t() {
        S{} = S{};
    }
}
