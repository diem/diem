address 0x0 {
module A {

    struct S has key {
        x: u64
    }
    public fun read_at(addr: address): u64 acquires S {
        let s = borrow_global<S>(addr);
        s.x
    }
    spec read_at {
        pragma opaque = true;
        aborts_if !exists<S>(addr);
        ensures result == global<S>(addr).x;
    }

    public fun mutate_at(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }
    spec mutate_at {
        pragma opaque = true;
    }
}

module B {
    use 0x0::A;

    public fun mutate_S_test1_incorrect(addr: address) {
        A::mutate_at(addr);
    }
    spec mutate_S_test1_incorrect {
        pragma opaque = true;
        modifies global<A::S>(addr);
    }

    public fun read_S_test1(addr: address): u64 {
        A::read_at(addr)
    }
    spec read_S_test1 {
        pragma opaque = true;
        modifies global<A::S>(addr);
    }

    public fun read_S_test2(addr: address): u64 {
        A::read_at(addr)
    }
    spec read_S_test1 {
        pragma opaque = true;
    }
}
}
