module M {
    resource struct R {}

    t0() {
        _ = R{};
        (_, _) = (R{}, R{});
    }
}
