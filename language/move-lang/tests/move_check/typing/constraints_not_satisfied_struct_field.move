module M {
    struct CupC<T: copy> {}
    struct R {}

    struct B {
        f: CupC<R>,
    }
}
