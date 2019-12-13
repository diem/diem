module M {
    struct S<T> { f: T<u64> }
    foo<T>(x: T<bool>): T<u64> {}
}
