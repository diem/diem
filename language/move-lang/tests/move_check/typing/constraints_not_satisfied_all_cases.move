module 0x8675309::M {
    struct CupR<T: key> { f: T }
    struct CupC<T: copy> { f: T }
    struct R has key {}
    struct C has copy {}

    fun no_constraint<T>(_c: CupC<T>, _r: CupR<T>) { abort 0 }

    fun t_resource<T: key>(_c: CupC<T>, _r: CupR<T>) { abort 0 }

    fun t_copyable<T: copy>(_c: CupC<T>, _r: CupR<T>) { abort 0 }

    fun r(_c: CupC<R>, _r: CupR<R>) { abort 0 }

    fun c(_c: CupC<C>, _r: CupR<C>) { abort 0 }
}
