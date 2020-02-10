module M {
    struct Box<T> { f: T }

    fun t0() {
        Box { f: () };
    }
}
