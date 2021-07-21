module 0x8675309::M {
    struct CupC<T: copy> { f: T }
    struct R {}

    struct B {
        f: CupC<R>,
    }
}
