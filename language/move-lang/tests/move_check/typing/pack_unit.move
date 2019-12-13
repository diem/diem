module M {
    struct Box<T> { f: T }

    t0() {
        Box { f: () };
    }
}
