module M {
    struct S has drop {}
    struct R {}

    fun t() {
        let u = 0;
        let s = S{};
        let r = R{};
        (u: u64);
        (s: S);
        R{} = (r: R);
    }
}
