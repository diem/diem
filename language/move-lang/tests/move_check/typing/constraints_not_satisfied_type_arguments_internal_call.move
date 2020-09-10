module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    fun box<T>() {
    }

    fun foo() {
        box<CupC<R>>();
    }

}
