module M {
    struct S<T> has drop {}

    fun t() {
        S{};
    }
}
