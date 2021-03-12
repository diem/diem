module 0x8675309::M {
    struct CupC<T: copy> {}
    struct R {}

    fun box<T>() {
    }

    fun foo() {
        box<CupC<R>>();
    }

}
