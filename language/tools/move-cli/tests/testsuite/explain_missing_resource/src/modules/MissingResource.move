address 0x2 {
module MissingResource {
    struct R has key { }

    public fun f() acquires R {
        borrow_global<R>(@0x0);
    }
}
}
