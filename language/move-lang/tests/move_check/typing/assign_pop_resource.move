module 0x8675309::M {
    struct R {}

    fun t0() {
        _ = R{};
        (_, _) = (R{}, R{});
    }
}
