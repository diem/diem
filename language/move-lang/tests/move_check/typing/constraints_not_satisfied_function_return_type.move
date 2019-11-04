module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    foo():  CupC<R> {
        abort 0
    }
}
