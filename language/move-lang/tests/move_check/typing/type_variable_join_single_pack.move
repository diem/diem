module M {
    struct Box<T> { f1: T, f2: T }

    fun t0() {
        let b = Box { f1: 0, f2: 1 };
        (*&b: Box<u64>);
        let b2 = Box { f1: *&b, f2: b };
        (b2: Box<Box<u64>>);
    }
}
