address 0x0 {
module A {

    struct S has key {
        x: u64
    }
    spec schema ModifiesSchema {
        addr: address;
        modifies global<S>(addr);
    }

    public fun mutate_at(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }
    spec mutate_at {
        //pragma opaque = true;
        include ModifiesSchema;
    }

    public fun mutate_at_wrapper1(addr: address) acquires S {
        mutate_at(addr)
    }
    spec mutate_at_wrapper1 {
        pragma opaque = true;
        include ModifiesSchema;
    }

    public fun mutate_at_wrapper2(addr1: address, addr2: address) acquires S {
        mutate_at(addr1);
        mutate_at(addr2)
    }
    spec mutate_at_wrapper2 {
        pragma opaque = true;
        include ModifiesSchema{addr: addr1};
    }
}
}
