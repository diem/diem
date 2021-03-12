module 0x8675309::M {
    struct R {}

    fun t0() {
        let _: R = R{};
        // the following is an invalid binding too but its error message will
        // not show because the compilation fails early at the typing phase
        let _r: R = R{};
        let (_, _):(R, R) = (R{}, R{});
    }
}
