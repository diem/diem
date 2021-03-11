address 0x42 {
module M {

    struct Cup<T> has drop {}
    struct Pair<T1, T2> has drop {}
    struct R {}
    struct S has drop {}

    // Error on a type that can have the drop ability, but the instantiation does not
    fun t() {
        let x = Cup<R> {};
        &x;
        let x = Pair<S, R> {};
        &x;
    }
}
}
