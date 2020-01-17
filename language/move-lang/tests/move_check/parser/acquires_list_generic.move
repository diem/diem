module M {
    struct CupC<T: copyable> {}
    resource struct R {}
    resource struct B<T> {}

    foo() acquires B<CupC<R>> {
        abort 0
    }
}
