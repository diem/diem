module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    fun foo():  CupC<R> {
        abort 0
    }
}
