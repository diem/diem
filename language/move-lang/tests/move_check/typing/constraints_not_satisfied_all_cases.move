module M {
    struct CupR<T: key> {}
    struct CupC<T: copy> {}
    struct R has key {}
    struct C has copy {}

    fun no_constraint<T>(c: CupC<T>, r: CupR<T>) {}

    fun t_resource<T: key>(c: CupC<T>, r: CupR<T>) {}

    fun t_copyable<T: copy>(c: CupC<T>, r: CupR<T>) {}

    fun r(c: CupC<R>, r: CupR<R>) {}

    fun c(c: CupC<C>, r: CupR<C>) {}
}
