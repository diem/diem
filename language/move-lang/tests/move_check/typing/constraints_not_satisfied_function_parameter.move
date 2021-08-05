module 0x8675309::M {
    struct CupC<T: copy> { f: T }
    struct R {}

    fun foo(_x: CupC<R>) { abort 0 }
}
