module MissingResource {
    resource struct R { }

    public fun f() acquires R {
        borrow_global<R>(0x0);
    }
}
