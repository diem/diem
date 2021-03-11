address 0x42 {
module M {
    struct S has copy, drop, store {}
    struct R {}
    struct Box<T> has key, store {}
    struct Pair<T1, T2> has key, store {}
    struct K has key {}

    fun ignore<T>(x: T) {
        abort 0
    }

    // types that can have key/store but the specific instantiation does not
    fun ex<T>(s: &signer, a1: address) acquires Box, Pair {
        move_to(s, Box<R> {});
        borrow_global<Box<T>>(a1);
        borrow_global_mut<Box<Box<T>>>(a1);
        Pair {} = move_from<Pair<S, R>>(a1);
        exists<Pair<Box<T>, S>>(a1);

        borrow_global<Box<K>>(a1);
        borrow_global_mut<Pair<S, K>>(a1);
    }
}
}
