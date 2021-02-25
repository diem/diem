module M {
    struct S {}
    struct R {}

    fun t0() {
        &R{};
        &mut R{};
    }
}
