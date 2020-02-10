module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    struct B<T>{}

    fun foo() {
        let B<CupC<R>> {} = abort 0;
        B<CupC<R>> {} = abort 0;
    }
}
