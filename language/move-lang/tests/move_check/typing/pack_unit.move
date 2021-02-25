module M {
    struct Box<T> has drop { f: T }

    fun t0() {
        Box { f: () };
    }
}
