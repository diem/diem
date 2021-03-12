module 0x8675309::M {
    struct S<T, T> {}
    struct S2<T: drop, T: key, T> {}
    struct R<T, T> {}
    struct R2<T: drop, T: key, T> {}
}
