address 0x2 {

module X {
    struct R has key, store {}
}

module M {
    use 0x2::X;

    struct S has store {}
    struct R<T> has key {v: T}

    fun destroy<T: store>(account: &signer, v: T) {
        move_to(account, R { v })
    }

    fun t0<T: key>() acquires
        T,
        u64,
        X::R,
        S,
    {
    }

    fun any<T>(): T {
        abort 0
    }

    fun t1<T: key + store>(account: &signer, a: address) {
        destroy(account, move_from(a));
        destroy(account, move_from<T>(a));
        destroy(account, move_from<u64>(a));
        destroy(account, move_from<X::R>(a));
        destroy(account, move_from<S>(a));

        borrow_global(a);
        borrow_global<T>(a);
        borrow_global<u64>(a);
        borrow_global<X::R>(a);
        borrow_global<S>(a);

        borrow_global_mut(a);
        borrow_global_mut<T>(a);
        borrow_global_mut<u64>(a);
        borrow_global_mut<X::R>(a);
        borrow_global_mut<S>(a);

        exists(a);
        exists<T>(a);
        exists<u64>(a);
        exists<X::R>(a);
        exists<S>(a);

        move_to(account, any());
        move_to<T>(account, any());
        move_to<u64>(account, any());
        move_to<X::R>(account, any());
        move_to<S>(account, any());
    }
}

}
