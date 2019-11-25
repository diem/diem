module M {
    struct Box<T> { f1: T, f2: T }

    t0() {
        let b = Box { f1: false, f2: 1 };
        let b2 = Box { f1: Box { f1: 0, f2: 0 }, f2:  Box { f1: false, f2: false } };
    }
}
