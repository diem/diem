module 0x8675309::M {
    struct R<T> {}
    struct S {}
    struct Box<T> has drop {}

    fun t0() {
        (0, S{}, R<u64> {});
        (0, S{}, Box<R<u64>> {});
        (0, S{}, Box {});
    }

}
