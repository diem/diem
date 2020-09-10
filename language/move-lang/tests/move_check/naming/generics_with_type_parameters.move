module M {
    struct S<T> { f: T<u64> }
    fun foo<T>(x: T<bool>): T<u64> {}
}
