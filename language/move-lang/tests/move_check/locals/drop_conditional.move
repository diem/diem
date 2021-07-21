address 0x42 {
module M {

    struct Cup<T> has drop { f: T }
    struct Pair<T1, T2> has drop { f1: T1, f2: T2 }
    struct R {}
    struct S has drop {}

    // Error on a type that can have the drop ability, but the instantiation does not
    fun t() {
        let x = Cup<R> { f: R{} };
        &x;
        let x = Pair<S, R> { f1: S{}, f2: R{} };
        &x;
    }
}
}
