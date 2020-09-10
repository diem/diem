module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    resource struct B {
        f: CupC<R>,
    }
}
