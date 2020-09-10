module M {
    struct X<T> {}
    struct S { f: X<u64> }
    fun foo() {
        let x = S { f: X{} };
        x.f<u64>;
    }
}
