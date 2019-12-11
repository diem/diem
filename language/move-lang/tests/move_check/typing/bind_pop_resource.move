module M {
    resource struct R {}

    t0() {
        let _: R = R{};
        let _r: R = R{};
        let (_, _):(R, R) = (R{}, R{});
    }
}
