module M {
    fun foo<T, T>() {}
    fun foo2<T: drop, T: key, T>() {}
}
