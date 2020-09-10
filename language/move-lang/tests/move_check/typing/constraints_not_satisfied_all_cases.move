module M {
    struct CupR<T: resource> {}
    struct CupC<T: copyable> {}
    resource struct R {}
    struct C {}

    fun no_constraint<T>(c: CupC<T>, r: CupR<T>) {}

    fun t_resource<T: resource>(c: CupC<T>, r: CupR<T>) {}

    fun t_copyable<T: copyable>(c: CupC<T>, r: CupR<T>) {}

    fun r(c: CupC<R>, r: CupR<R>) {}

    fun c(c: CupC<C>, r: CupR<C>) {}
}
