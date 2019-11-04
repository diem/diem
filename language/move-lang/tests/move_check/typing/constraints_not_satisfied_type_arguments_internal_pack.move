module M {
    struct CupC<T: copyable> {}
    resource struct R {}

    struct Box<T> {}

    foo() {
        Box<CupC<R>>{};
        Box<R>{};
    }

}
