module 0x8675309::M {
    struct S<T> { f: T }
    struct R<T1, T2> { f1: T1, f2: T2 }

    fun foo<T>() {
        foo<R<u64, S<S<T>>>>()
    }
}
