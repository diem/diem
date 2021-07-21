module 0x8675309::M {
    struct R<T> { f: T }
    struct S {}
    struct Box<T> has drop { f: T }

    fun t0() {
        (0, S{ }, R<u64> { f: 1 });
        (0, S{ }, Box<R<u64>> { f: R { f: 1 } });
        (0, S{ }, Box { f: abort 0 });
    }

}
