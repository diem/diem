// flag: --v2
address 0x0 {
module A {
    resource struct S {
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
    spec fun mutate_at {
        //pragma opaque = true;
        include ModifiesSchema;
    }

    public fun mutate_at_wrapper1(addr: address) acquires S {
        mutate_at(addr)
    }
    spec fun mutate_at_wrapper1 {
        pragma opaque = true;
        include ModifiesSchema;
    }

    public fun mutate_at_wrapper2(addr1: address, addr2: address) acquires S {
        mutate_at(addr1);
        mutate_at(addr2)  // TODO(wrwg): the source position of those calls in the bytecode is broken
    }
    spec fun mutate_at_wrapper2 {
        pragma opaque = true;
        include ModifiesSchema{addr: addr1};
    }
}
}
