module M {
    struct Box<T> { f1: T, f2: T }

    fun new<T>(): Box<T> {
        abort 0
    }

    fun t0() {
        let Box { f1, f2 } = new();
        (f1: u64);
        (f2: u64);
        let Box { f1, f2 } = new();
        (f1: Box<u64>);
        (f2: Box<u64>);
    }
}
