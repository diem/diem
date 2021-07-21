module 0x8675309::M {
    struct R<phantom T> {}
    struct S {}

    fun t0(): (u64, S, R<u64>) {
        (0, (S{}, R{}))
    }

}
