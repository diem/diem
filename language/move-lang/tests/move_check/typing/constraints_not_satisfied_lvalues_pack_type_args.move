module M {
    struct CupC<T: copy> {}
    struct R {}

    struct B<T>{}

    fun foo() {
        let B<CupC<R>> {} = abort 0;
        B<CupC<R>> {} = abort 0;
    }
}
