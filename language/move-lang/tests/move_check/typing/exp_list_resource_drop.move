module M {
    resource struct R<T> {}
    struct S {}
    struct Box<T> {}

    fun t0() {
        (0, S{}, R<u64> {});
        (0, S{}, Box<R<u64>> {});
        (0, S{}, Box {});
    }

}
