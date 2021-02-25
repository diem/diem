module M {
    struct CupC<T: copy> {}
    struct R {}

    fun foo() {
        let x: CupC<R>;
    }

}
