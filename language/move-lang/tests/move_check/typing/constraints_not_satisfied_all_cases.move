module M {
    struct CupR<T: resource> {}
    struct CupC<T: copyable> {}
    resource struct R {}
    struct C {}

    no_constraint<T>(c: CupC<T>, r: CupR<T>) {}

    t_resource<T: resource>(c: CupC<T>, r: CupR<T>) {}

    t_copyable<T: copyable>(c: CupC<T>, r: CupR<T>) {}

    r(c: CupC<R>, r: CupR<R>) {}

    c(c: CupC<C>, r: CupR<C>) {}
}
