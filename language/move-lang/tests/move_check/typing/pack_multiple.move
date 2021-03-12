module 0x8675309::M {
    struct Box<T> has drop { f: T }

    fun t0() {
        Box { f: (0, 1) };
        Box { f: (0, 1, 2) };
        Box { f: (true, Box { f: 0 }) };
    }
}
