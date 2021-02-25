module M {
    struct CupC<T: copy> {}
    struct R {}

    fun foo(x: CupC<R>) {}
}
