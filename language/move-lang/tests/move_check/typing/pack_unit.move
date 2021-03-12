module 0x8675309::M {
    struct Box<T> has drop { f: T }

    fun t0() {
        Box { f: () };
    }
}
