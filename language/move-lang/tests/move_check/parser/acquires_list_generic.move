module M {
    struct CupC<T: drop> {}
    struct R {}
    struct B<T> {}

    fun foo() acquires B<CupC<R>> {
        abort 0
    }
}
