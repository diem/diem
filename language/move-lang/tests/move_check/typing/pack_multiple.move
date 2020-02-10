module M {
    struct Box<T> { f: T }

    fun t0() {
        Box { f: (0, 1) };
        Box { f: (0, 1, 2) };
        Box { f: (true, Box { f: 0 }) };
    }
}
