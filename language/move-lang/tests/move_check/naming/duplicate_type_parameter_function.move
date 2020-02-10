module M {
    fun foo<T, T>() {}
    fun foo2<T: copyable, T: resource, T>() {}
}
