address 0x42 {
module M {
    struct S has copy, drop, store {}
    struct R {}
    struct Box<T> has drop { f: T }
    struct Pair<T1, T2> has drop { f1: T1, f2: T2 }

    // types that can have drop but the specific instantiation does not
    fun ex<T: copy>(t: T) {
        Box<R> { f: R{} };
        Box<Box<R>> { f: Box { f: R{} } };
        Box<T> { f: t };
        Box<Box<T>> { f: Box { f: t } };
        Pair<S, R> { f1: S{}, f2: R{} };
        (Pair<S, R> { f1: S{}, f2: R{} }, 0, @0x1);

        Box<R> { f: R {} } == Box<R> { f: R {} };
        Box<Box<R>> { f: Box { f: R {} } } == Box<Box<R>> { f: Box { f: R {} }};
        Box<T> { f: t } == Box<T> { f: t };
        Box<Box<T>> { f: Box { f: t } } == Box<Box<T>> { f: Box { f: t} };
        Pair<R, S> { f1: R{}, f2: S{} } == Pair<R, S> { f1: R{}, f2: S{} };
    }
}
}
