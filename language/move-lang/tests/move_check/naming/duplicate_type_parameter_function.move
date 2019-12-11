module M {
    foo<T, T>() {}
    foo2<T: copyable, T: resource, T>() {}
}
