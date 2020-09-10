// Good: two instances foo<T> & foo<u64> (if T != u64) for any T::

module M {
    fun f<T>() {
        f<u64>()
    }
}
