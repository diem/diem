address 0x1:

module X {
    resource struct R{}
}

module M {
    use 0x1::X;

    struct S {}
    resource struct R<T> {v: T}

    destroy<T>(v: T) {
        move_to_sender(R { v })
    }

    t0<T: resource>() acquires
        T,
        u64,
        X::R,
        S,
        R<u64>,
        R<T>
    {
    }

    any<T>(): T {
        abort 0
    }

    t1<T: resource>(a: address) {
        destroy(move_from(a));
        destroy(move_from<T>(a));
        destroy(move_from<u64>(a));
        destroy(move_from<X::R>(a));
        destroy(move_from<S>(a));
        destroy(move_from<R<u64>>(a));
        destroy(move_from<R<T>>(a));

        borrow_global(a);
        borrow_global<T>(a);
        borrow_global<u64>(a);
        borrow_global<X::R>(a);
        borrow_global<S>(a);
        borrow_global<R<u64>>(a);
        borrow_global<R<T>>(a);

        borrow_global_mut(a);
        borrow_global_mut<T>(a);
        borrow_global_mut<u64>(a);
        borrow_global_mut<X::R>(a);
        borrow_global_mut<S>(a);
        borrow_global_mut<R<u64>>(a);
        borrow_global_mut<R<T>>(a);

        exists(a);
        exists<T>(a);
        exists<u64>(a);
        exists<X::R>(a);
        exists<S>(a);
        exists<R<u64>>(a);
        exists<R<T>>(a);

        move_to_sender(any());
        move_to_sender<T>(any());
        move_to_sender<u64>(any());
        move_to_sender<X::R>(any());
        move_to_sender<S>(any());
        move_to_sender<R<u64>>(any());
        move_to_sender<R<T>>(any());
    }
}
