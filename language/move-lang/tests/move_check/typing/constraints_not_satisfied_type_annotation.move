module M {
    struct CupC<T: copyable> {}
    struct C {}
    resource struct R {}

    fun foo() {
        ignore((abort 0: CupC<R>));
    }

    fun ignore<T>(x: T) {

    }

}
