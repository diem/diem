module M {
    struct S has drop {}
    struct R {}

    fun t() {
        let u = 0;
        let s = S{};
        let r = R{};
        (move u: u64);
        (move s: S);
        R{} = (move r: R);
    }
}
