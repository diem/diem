module M {
    resource struct R {}

    fun t0() {
        _ = R{};
        (_, _) = (R{}, R{});
    }
}
