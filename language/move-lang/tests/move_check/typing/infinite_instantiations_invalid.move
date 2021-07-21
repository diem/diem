address 0x42 {

module X {
    struct Box<T> { f: T }

    public fun t<T>() {
        t<Box<T>>()
    }

    public fun x<T>() {
        y<Box<T>>()
    }
    public fun y<T>() {
        x<Box<T>>()
    }

    public fun a<A>() {
        b<A>()
    }
    public fun b<B>() {
        c<B>()
    }
    public fun c<C>() {
        a<Box<C>>()
    }
}

module Y {
    struct Box<T> { f: T }

    public fun x<T>() {
        y<Box<T>>()
    }
    public fun y<T>() {
        z<Box<T>>()
    }
    public fun z<T>() {
        z<Box<T>>()
    }

    public fun a<A>() {
        b<A>()
    }
    public fun b<B>() {
        c<B>()
    }
    public fun c<C>() {
        d<Box<C>>()
    }
    public fun d<D>() {
        a<D>()
    }
}

module Z {
    struct Box<T> { f: T }

    public fun tl<TL>() {
        tr<TL>()
    }
    public fun tr<TR>() {
        bl<Box<TR>>();
        br<TR>()
    }
    public fun br<BR>() {
        bl<BR>()
    }
    public fun bl<BL>() {
        tl<BL>()
    }
}

}
