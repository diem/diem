module M {
    resource struct R<T> {}
    struct S {}

    t0(): (u64, S, R<u64>) {
        (0, (S{}, R{}))
    }

}
