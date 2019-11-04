module M {
    struct S {}
    resource struct R {}

    t() {
        let u = 0;
        let s = S{};
        let r = R{};
        (u: u64);
        (s: S);
        R{} = (r: R);
    }
}
