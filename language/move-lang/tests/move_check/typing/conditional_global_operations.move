address 0x42 {
module M {
    struct S has copy, drop, store {}
    struct R has drop {}
    struct Box<T> has key, store { f: T }
    struct Pair<T1, T2> has key, store { f1: T1, f2: T2 }
    struct K has key {}

    fun ignore<T>(_x: T) {
        abort 0
    }

    // types that can have key/store but the specific instantiation does not
    fun ex<T>(s: &signer, a1: address) acquires Box, Pair {
        move_to(s, Box<R> { f: R {} });
        borrow_global<Box<T>>(a1);
        borrow_global_mut<Box<Box<T>>>(a1);
        Pair { f1: _, f2: _ } = move_from<Pair<S, R>>(a1);
        exists<Pair<Box<T>, S>>(a1);

        borrow_global<Box<K>>(a1);
        borrow_global_mut<Pair<S, K>>(a1);
    }
}
}
