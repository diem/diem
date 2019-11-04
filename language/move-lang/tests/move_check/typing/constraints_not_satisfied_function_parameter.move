module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    foo(x: CupC<R>) {}
}
