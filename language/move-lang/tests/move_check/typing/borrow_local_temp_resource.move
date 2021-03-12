module 0x8675309::M {
    struct S {}
    struct R {}

    fun t0() {
        &R{};
        &mut R{};
    }
}
