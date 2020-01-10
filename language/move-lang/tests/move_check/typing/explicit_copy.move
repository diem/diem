module M {
    struct S {}
    resource struct R {}

    t() {
        let u = 0;
        let s = S{};
        (copy u: u64);
        (copy s: S);
        s;
        u;
    }
}
