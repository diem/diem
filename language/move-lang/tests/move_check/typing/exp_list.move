module 0x8675309::M {
    struct R<T> { f: T }
    struct S {}

    fun t0(): (u64, S, R<R<u64>>) {
        (0, S{}, R{ f: R { f: 1 } })
    }

    fun t1(s: &S, r: &mut R<u64>): (u64, &S, &mut R<u64>) {
        (0, s, r)
    }
}
