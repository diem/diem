// Good: two instances foo<T> & foo<u64> (if T != u64) for any T::

module 0x8675309::M {
    fun f<T>() {
        f<u64>()
    }
}
