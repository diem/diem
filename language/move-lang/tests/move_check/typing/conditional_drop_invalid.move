address 0x42 {
module M {
    struct S has copy, drop, store {}
    struct R {}
    struct Box<T> has drop {}
    struct Pair<T1, T2> has drop {}

    // types that can have drop but the specific instantiation does not
    fun ex<T>() {
        Box<R> {};
        Box<Box<R>> {};
        Box<T> {};
        Box<Box<T>> {};
        Pair<S, R> {};
        (Pair<S, R> {}, 0, 0x1);

        Box<R> {} == Box<R> {};
        Box<Box<R>> {} == Box<Box<R>> {};
        Box<T> {} == Box<T> {};
        Box<Box<T>> {} == Box<Box<T>> {};
        Pair<R, S> {} == Pair<R, S> {};
    }
}
}
