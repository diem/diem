address 0x42 {
module M {
    struct S has copy, drop, store {}
    struct R {}
    struct Box<T> has copy { f: T }
    struct Pair<T1, T2> has copy { f1: T1, f2: T2}

    fun ignore<T>(_x: T) {
        abort 0
    }

    // types that can have copy but the specific instantiation does not
    fun ex<T>(t1: T, t2: T, t3: T, t4: T) {
        let x = Box<R> { f: R{} };
        ignore(copy x);
        let x = Box<Box<R>> { f: Box { f: R{} } };
        ignore(copy x);
        let x = Box<T> { f: t1 };
        ignore(copy x);
        let x = Box<Box<T>> { f: Box { f: t2 } };
        ignore(copy x);
        let x = Pair<S, R> { f1: S{}, f2: R{} };
        ignore(copy x);


        let x = &Box<R> { f: R{} };
        ignore(*x);
        let x = &Box<Box<R>> { f: Box { f: R{} } };
        ignore(*x);
        let x = &Box<T> { f: t3 };
        ignore(*x);
        let x = &Box<Box<T>> { f: Box { f: t4 } };
        ignore(*x);
        let x = &Pair<R, S> { f1: R{}, f2: S{} };
        ignore(*x);

        abort 0
    }
}
}
