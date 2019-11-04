module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    box<T>() {
    }

    foo() {
        box<CupC<R>>();
    }

}
