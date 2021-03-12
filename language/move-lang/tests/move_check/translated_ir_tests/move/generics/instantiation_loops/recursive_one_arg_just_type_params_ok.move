// This is good as there is only one instance foo<T> for any T::

module 0x8675309::M {
    public fun f<T>(x: T) {
        f<T>(x)
    }
}
