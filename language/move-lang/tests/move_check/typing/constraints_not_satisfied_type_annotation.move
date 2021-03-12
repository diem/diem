module 0x8675309::M {
    struct CupC<T: copy> {}
    struct C {}
    struct R {}

    fun foo() {
        ignore((abort 0: CupC<R>));
    }

    fun ignore<T>(x: T) {

    }

}
