module M {
    struct S has copy, drop {}
    struct R {}

    fun t() {
        let u = 0;
        let s = S{};
        (copy u: u64);
        (copy s: S);
        s;
        u;
    }
}
