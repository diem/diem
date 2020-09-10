module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    fun foo(x: CupC<R>) {}
}
