module 0x8675309::M {
    fun foo<T, T>() {}
    fun foo2<T: drop, T: key, T>() {}
}
