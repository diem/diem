address 0x1 {
module BoolFootprint {

    struct B has key { b: bool }

    public fun global_get(a: address): bool acquires B {
        exists<B>(a) && borrow_global<B>(a).b
    }

    public fun call_get(a: address) acquires B {
        assert(global_get(a), 22)
    }
}
}
