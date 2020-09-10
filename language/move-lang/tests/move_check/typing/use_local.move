module M {
    struct S {}
    resource struct R {}

    fun t() {
        let u = 0;
        let s = S{};
        let r = R{};
        (u: u64);
        (s: S);
        R{} = (r: R);
    }
}
