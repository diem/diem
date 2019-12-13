module M {
    struct X<T> {}
    struct S { f: X<u64> }
    foo() {
        let x = S { f: X{} };
        x.f<u64>;
    }
}
