module M {
    struct S<T, T> {}
    struct S2<T: copyable, T: resource, T> {}
    struct R<T, T> {}
    struct R2<T: copyable, T: resource, T> {}
}
