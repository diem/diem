module 0x8675309::M {
    struct S<T> { f: T<u64> }
    fun foo<T>(x: T<bool>): T<u64> {}
}
