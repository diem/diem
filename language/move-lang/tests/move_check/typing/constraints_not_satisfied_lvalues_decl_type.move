module 0x8675309::M {
    struct CupC<T: copy> { f: T }
    struct R {}

    fun foo() {
        let _x: CupC<R>;
    }

}
