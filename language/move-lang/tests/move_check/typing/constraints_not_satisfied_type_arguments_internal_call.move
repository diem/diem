module M {
    struct CupC<T: copy> {}
    struct R {}

    fun box<T>() {
    }

    fun foo() {
        box<CupC<R>>();
    }

}
