module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    fun foo() {
        let x: CupC<R>;
    }

}
