address 0x123 {
module Test {

    struct A<T1: store, T2: store> has key, store {
        x1: T1,
        x2: T2,
    }

    public fun test<T: store>(): bool {
        exists<A<u64, T>>(@0x1)
    }

    public fun update<T: drop + store>(x: T) acquires A {
        borrow_global_mut<A<T, T>>(@0x1).x1 = x;
    }

    public fun update_caller() acquires A {
        update<u8>(1)
    }

    public fun update_ints() acquires A {
        borrow_global_mut<A<u64, u128>>(@0x1).x1 = 22;
    }

    public fun publish<T: store>(signer: &signer, x: A<T, u8>) {
        move_to<A<T, u8>>(signer, x)
    }
}
}
