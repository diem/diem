module M {
    struct CupC<T: copyable> {}
    struct C {}
    resource struct R {}

    foo() {
        ignore((abort 0: CupC<R>));
    }

    ignore<T>(x: T) {

    }

}
