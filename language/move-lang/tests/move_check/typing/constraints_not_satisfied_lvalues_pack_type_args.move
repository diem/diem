module 0x8675309::M {
    struct CupC<phantom T: copy> {}
    struct R {}

    struct B<phantom T>{}

    fun foo() {
        let B<CupC<R>> {} = abort 0;
        B<CupC<R>> {} = abort 0;
    }
}
