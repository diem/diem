module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    foo() {
        let x: CupC<R> = abort 0;
    }
}
