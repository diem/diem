module M {
    resource struct R<T> {}
    struct S {}

    t0(): (u64, S, R<R<u64>>) {
        (0, S{}, R{})
    }

    t1(s: &S, r: &mut R<u64>): (u64, &S, &mut R<u64>) {
        (0, s, r)
    }
}
