address 0x42 {
module M {
    struct S has copy, drop, store {}
    struct R {}
    struct Box<T> has copy {}
    struct Pair<T1, T2> has copy {}

    fun ignore<T>(x: T) {
        abort 0
    }

    // types that can have copy but the specific instantiation does not
    fun ex<T>() {
        let x = Box<R> {};
        ignore(copy x);
        let x = Box<Box<R>> {};
        ignore(copy x);
        let x = Box<T> {};
        ignore(copy x);
        let x = Box<Box<T>> {};
        ignore(copy x);
        let x = Pair<S, R> {};
        ignore(copy x);


        let x = &Box<R> {};
        ignore(*x);
        let x = &Box<Box<R>> {};
        ignore(*x);
        let x = &Box<T> {};
        ignore(*x);
        let x = &Box<Box<T>> {};
        ignore(*x);
        let x = &Pair<R, S> {};
        ignore(*x);
    }
}
}
