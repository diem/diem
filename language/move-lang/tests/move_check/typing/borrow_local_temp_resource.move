module M {
    struct S {}
    resource struct R {}

    fun t0() {
        &R{};
        &mut R{};
    }
}
