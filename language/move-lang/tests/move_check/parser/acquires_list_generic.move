module M {
    struct CupC<T: copyable> {}
    resource struct R {}
    resource struct B<T> {}

    fun foo() acquires B<CupC<R>> {
        abort 0
    }
}
