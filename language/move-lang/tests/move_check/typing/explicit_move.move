module M {
    struct S {}
    resource struct R {}

    t() {
        let u = 0;
        let s = S{};
        let r = R{};
        (move u: u64);
        (move s: S);
        R{} = (move r: R);
    }
}
