module M {
    struct S {}
    resource struct R {}

    t0() {
        &R{};
        &mut R{};
    }
}
