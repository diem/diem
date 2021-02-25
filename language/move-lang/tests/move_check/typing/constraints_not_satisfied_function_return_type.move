module M {
    struct CupC<T: copy> {}
    struct R {}

    fun foo():  CupC<R> {
        abort 0
    }
}
